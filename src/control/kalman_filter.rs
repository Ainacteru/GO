use core::{f32, todo};

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use defmt::info;
use embassy_time::Instant;
use uom::si::{acceleration, f32::{Acceleration, Length, Velocity}, length::{self}, velocity};

use crate::{control::error::KalmanFilterError::{self}, sensors::{bmp::Bmp, imu::Imu}, util::math::matrix::{Matrix, matrix3x1::{Matrix1x3, Matrix3x1}, matrix3x3::Matrix3x3}};
use micromath::{Quaternion, vector::F32x3};

struct AltitudeEstimation {
    height: Length,
    vertical_velocity: Velocity,
    accel_bias: Acceleration,
    error_covariance: Matrix3x3,
}

struct OrientationEstimation {
    state_estimation: Quaternion,
    error_covariance: Matrix3x3,
    antiparallel_count: u32,
}

pub struct KalmanFilter <B: I2c<SevenBitAddress>, D: DelayNs> {
    imu: Imu<B, D>,
    baro: Bmp<B, D>,

    prev_time: Instant,
    orientation_estimation: OrientationEstimation,
    alt_state_estimation: AltitudeEstimation,
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {
    pub fn new(imu: Imu<B, D>, baro: Bmp<B, D>) -> Self {
        Self {
            imu,
            baro,

            prev_time: Instant::now(),

            orientation_estimation: OrientationEstimation { 
                state_estimation: Quaternion::IDENTITY,
                error_covariance: Matrix3x3::new_diagonal([0.01, 0.01, 0.01]),
                antiparallel_count: 0, 
            },

            alt_state_estimation: AltitudeEstimation { 
                height: Length::new::<length::meter>(0.0), 
                vertical_velocity: Velocity::new::<velocity::meter_per_second>(0.0),
                accel_bias: Acceleration::new::<acceleration::meter_per_second_squared>(0.0),
                error_covariance: Matrix3x3::new_diagonal([1.0, 1.0, 1.0]),
            },

        }
    }
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {
    pub async fn calculate_state(&mut self) -> Result<(), KalmanFilterError> {

        self.predict().await?;
        self.correct().await?;

        Ok(())
    }

    pub async fn iir_filter(&mut self) -> Result<(), KalmanFilterError> {
        todo!()
    }

    pub async fn predict(&mut self) -> Result<(), KalmanFilterError> {
        // init
        const DEG2RAD: f32 = f32::consts::PI / 180.0;
        let gyro =  self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr)?;
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr)?;

        let a_world = self.orientation_estimation.state_estimation.rotate(accel);
        let measured_accel = (a_world.z - 1.0) * 9.80665;

        // ORIENTATION
        let w = F32x3 {
          x: gyro.x * DEG2RAD,
          y: gyro.y * DEG2RAD,
          z: gyro.z * DEG2RAD,
        };

        let omega = Quaternion::new(0.0, w.x, w.y, w.z);

        let q_dot = 0.5 * self.orientation_estimation.state_estimation * omega; // f(x, u)

        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_micros() as f32 / 1000000.0;
        let dt = dt.min(0.05);
        self.prev_time = now;

        //euler integrrration yay
        self.orientation_estimation.state_estimation = Self::normalize_exact(self.orientation_estimation.state_estimation + q_dot * dt);

        // error covariance matrix update

        let skew = Matrix3x3::from_array([
            [0.0,  -w.z,  w.y],
            [w.z,   0.0, -w.x],
            [-w.y,  w.x,  0.0],
        ]);

        // F = I - skew * dt
        let f = Matrix3x3::IDENTITY - skew * dt;
        // transpose F
        // Q
        // sec/hz to rads/sec
        const GYRO_VARIANCE: f32 = 9e-6;
        let q_value = GYRO_VARIANCE;

        let q_noise = Matrix3x3::new_diagonal(
            [q_value, q_value, q_value]
        );

        // P = FPF^T + Q
        self.orientation_estimation.error_covariance = f * self.orientation_estimation.error_covariance * f.transpose() + q_noise;

        //ALTITUDE

        // F
        let f = Matrix3x3::from_array([
            [1.0, dt, -(dt * dt) * 0.5],
            [0.0, 1.0, -dt],
            [0.0, 0.0, 1.0],
        ]);

        let b = Matrix3x1::from_array([0.5 * dt * dt, dt, 0.0]);

        let x = Matrix3x1::from_array([
            self.alt_state_estimation.height.get::<length::meter>(),
            self.alt_state_estimation.vertical_velocity.get::<velocity::meter_per_second>(),
            self.alt_state_estimation.accel_bias.get::<acceleration::meter_per_second_squared>(),
        ]);
        let x_new = f * x + b * measured_accel;

        self.alt_state_estimation.height = Length::new::<length::meter>(x_new.get(0));
        self.alt_state_estimation.vertical_velocity = Velocity::new::<velocity::meter_per_second>(x_new.get(1));
        self.alt_state_estimation.accel_bias = Acceleration::new::<acceleration::meter_per_second_squared>(x_new.get(2));
        
        const ACCEL_VAR: f32 = 0.05;
        const BIAS_VAR: f32 = 1e-4;
        let q = (b * b.transpose()) * ACCEL_VAR + Matrix3x3::new_diagonal([0.0, 0.0, BIAS_VAR * dt]);

        // P = FPF^T + Q
        self.alt_state_estimation.error_covariance = f * self.alt_state_estimation.error_covariance * f.transpose() + q;

        Ok(())
    }

    pub async fn correct(&mut self) -> Result<(), KalmanFilterError> {
        // init
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr)?;
        let gyro = self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr)?;

        let Some(baro_alt) = 
        self.baro.get_altitude().await.map_err(KalmanFilterError::BarometerErr)? else {
            return Ok(())
        };
        let baro_alt = baro_alt.get::<length::meter>();
        let alt_est = self.alt_state_estimation.height.get::<length::meter>();

        //orientation

        let mag = libm::sqrtf(accel.x*accel.x + accel.y*accel.y + accel.z*accel.z);

        if !(0.90..=1.15).contains(&mag) { // ouuuu
            return Ok(())
        }

        // check if board is moving too fast
        const MAX_RATE_FOR_CORRECTION: f32 = 60.0; // degs /s
        let rate = libm::sqrtf(gyro.x*gyro.x + gyro.y*gyro.y + gyro.z*gyro.z);
        
        if rate > MAX_RATE_FOR_CORRECTION {
            return Ok(()); // use the gyro only
        }

        let accel = F32x3 { x: accel.x / mag, y: accel.y / mag, z: accel.z / mag };

        let gravity = F32x3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };

        // R(q)
        let prediction = self.orientation_estimation.state_estimation.conj().rotate(gravity);

        let dot = accel.x*prediction.x + accel.y*prediction.y + accel.z*prediction.z;
  
        if dot < 0.0 {
            // only reset when genuinely still AND the accel really looks like pure gravity
            if rate < 5.0 && (mag - 1.0).abs() < 0.02 {
                self.orientation_estimation.antiparallel_count += 1;
                if self.orientation_estimation.antiparallel_count > 20 {          // ~0.3 s of agreement
                    let fix = Quaternion::from_two_vectors(prediction, accel);
                    self.orientation_estimation.state_estimation = Self::normalize_exact(self.orientation_estimation.state_estimation * fix);
                    self.orientation_estimation.error_covariance = Matrix3x3::new_diagonal([0.01, 0.01, 0.01]);
                    self.orientation_estimation.antiparallel_count = 0;
                }
            } else {
                self.orientation_estimation.antiparallel_count = 0;
            }
            return Ok(());
        }
        self.orientation_estimation.antiparallel_count = 0;

        let innovation = accel - prediction;

        let h = Matrix3x3::from_array( [
            [0.0, -prediction.z, prediction.y],
            [prediction.z, 0.0, -prediction.x],
            [-prediction.y, prediction.x, 0.0],
        ] );

        const ACCEL_VAR: f32 = 0.01;
        let accel_noise = Matrix3x3::new_diagonal([ACCEL_VAR, ACCEL_VAR, ACCEL_VAR]);

        let s = h * self.orientation_estimation.error_covariance * h.transpose() + accel_noise;
        let kalman_gain = self.orientation_estimation.error_covariance * h.transpose() * s.inverse().map_err(KalmanFilterError::Matrix)?;

        let correction = kalman_gain * innovation;

        let n = libm::sqrtf(correction.x*correction.x + correction.y*correction.y + correction.z*correction.z);
        let max_corr = 0.7_f32;
        let correction = if n > max_corr {
            F32x3 { x: correction.x*max_corr/n, y: correction.y*max_corr/n, z: correction.z*max_corr/n }
        } else { correction };

        let dq = Self::normalize_exact(Quaternion::new(1.0, correction.x/2.0, correction.y/2.0, correction.z/2.0));

        self.orientation_estimation.state_estimation = Self::normalize_exact(self.orientation_estimation.state_estimation * dq);

        let i_kh = Matrix3x3::IDENTITY - kalman_gain * h;
        self.orientation_estimation.error_covariance = i_kh * self.orientation_estimation.error_covariance * i_kh.transpose() + kalman_gain * accel_noise * kalman_gain.transpose();

        //height

        let h = Matrix1x3::from_array([1.0, 0.0, 0.0]);

        let y = baro_alt - alt_est;

        const BARO_VAR: f32 = 0.0056;

        let s = h * self.alt_state_estimation.error_covariance * h.transpose() + BARO_VAR;

        let k = (self.alt_state_estimation.error_covariance * h.transpose()) * (1.0 / s);

        let x = Matrix3x1::from_array([
            self.alt_state_estimation.height.get::<length::meter>(),
            self.alt_state_estimation.vertical_velocity.get::<velocity::meter_per_second>(),
            self.alt_state_estimation.accel_bias.get::<acceleration::meter_per_second_squared>(),
        ]);

        let x_new = x + k * y;
        self.alt_state_estimation.height = Length::new::<length::meter>(x_new.get(0));
        self.alt_state_estimation.vertical_velocity = Velocity::new::<velocity::meter_per_second>(x_new.get(1));
        self.alt_state_estimation.accel_bias = Acceleration::new::<acceleration::meter_per_second_squared>(x_new.get(2));

        // P = (I - K*H) P (I - K*H)^T + K*R*K^T
        let i_kh = Matrix3x3::IDENTITY - k * h;
        self.alt_state_estimation.error_covariance =
            i_kh * self.alt_state_estimation.error_covariance * i_kh.transpose()
            + (k * BARO_VAR) * k.transpose();

        Ok(())
    }
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {
    pub async fn imu_dat(&mut self) {
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr).unwrap();
        let gyro = self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr).unwrap();


        info!("accel xyz: {} {} {}", accel.x, accel.y, accel.z);
        info!("gyro xyz: {} {} {}", gyro.x, gyro.y, gyro.z);

    }

    /// Returns the atitude state 
    pub fn atitude(&self) -> Quaternion {
        self.orientation_estimation.state_estimation
    }

    pub fn altitude(&self) -> Length {
        self.alt_state_estimation.height
    }
    pub fn vertical_velocity(&self) -> Velocity {
        self.alt_state_estimation.vertical_velocity
    }

    /// using libm instead of micromath's normalize 
    fn normalize_exact(q: Quaternion) -> Quaternion {
        let n = libm::sqrtf(q.norm());
        if n == 0.0 { return Quaternion::IDENTITY; }
        q.scale(1.0 / n)
    }
}