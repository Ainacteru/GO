use core::f32;

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use defmt::info;
use embassy_time::Instant;

use crate::{control::error::KalmanFilterError, sensors::imu::Imu, util::math::matrix::{Matrix, matrix3x3::Matrix3x3}};
use micromath::{F32Ext, Quaternion, vector::{F32x3, Vector}};

pub struct KalmanFilter <B: I2c<SevenBitAddress>, D: DelayNs> {
    imu: Imu<B, D>,
    prev_time: Instant,
    orien_state_estimation: Quaternion,
    orien_error_covariance: Matrix3x3,
    orien_antiparallel_count: u32,
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {

    pub fn new(imu: Imu<B, D>) -> Self {
        Self {
            imu,
            prev_time: Instant::now(),
            orien_state_estimation: Quaternion::IDENTITY,
            orien_error_covariance: Matrix3x3::new_diagonal([0.01, 0.01, 0.01]),
            orien_antiparallel_count: 0,
        }
    }

    pub async fn calc_atitude(&mut self) -> Result<(), KalmanFilterError> {

        //need an imu low pass filter
        self.iir_filter().await?;

        self.atitude_predict().await?;

        self.atitude_correct().await?;

        Ok(())
    }

    pub async fn calc_altitude(&mut self) -> Result<(), KalmanFilterError> {

        self.altitude_correct().await?;
        self.altitude_predict().await?;

        Ok(())
    }

    async fn altitude_predict(&mut self) -> Result<(), KalmanFilterError> {
        Ok(())
    }

    async fn altitude_correct(&mut self) -> Result<(), KalmanFilterError> {
        Ok(())
    }

    async fn atitude_predict(&mut self) -> Result<(), KalmanFilterError> {
        const DEG2RAD: f32 = f32::consts::PI / 180.0;
        let gyro =  self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr)?;

        let w = F32x3 {
          x: gyro.x * DEG2RAD,
          y: gyro.y * DEG2RAD,
          z: gyro.z * DEG2RAD,
        };

        let omega = Quaternion::new(0.0, w.x, w.y, w.z);

        let q_dot = 0.5 * self.orien_state_estimation * omega; // f(x, u)

        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_micros() as f32 / 1000000.0;
        let dt = dt.min(0.05);
        self.prev_time = now;

        self.orien_state_estimation = Self::normalize_exact(self.orien_state_estimation + q_dot * dt);

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
        self.orien_error_covariance = f * self.orien_error_covariance * f.transpose() + q_noise;

        Ok(())
    }

    async fn atitude_correct(&mut self) -> Result<(), KalmanFilterError> {
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr)?;
        let gyro = self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr)?;

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
        let prediction = self.orien_state_estimation.conj().rotate(gravity);

        let dot = accel.x*prediction.x + accel.y*prediction.y + accel.z*prediction.z;
  
        if dot < 0.0 {
            // only reset when genuinely still AND the accel really looks like pure gravity
            if rate < 5.0 && (mag - 1.0).abs() < 0.02 {
                self.orien_antiparallel_count += 1;
                if self.orien_antiparallel_count > 20 {          // ~0.3 s of agreement
                    let fix = Quaternion::from_two_vectors(prediction, accel);
                    self.orien_state_estimation = Self::normalize_exact(self.orien_state_estimation * fix);
                    self.orien_error_covariance = Matrix3x3::new_diagonal([0.01, 0.01, 0.01]);
                    self.orien_antiparallel_count = 0;
                }
            } else {
                self.orien_antiparallel_count = 0;
            }
            return Ok(());
        }
        self.orien_antiparallel_count = 0;

        let innovation = accel - prediction;

        let h = Matrix3x3::from_array( [
            [0.0, -prediction.z, prediction.y],
            [prediction.z, 0.0, -prediction.x],
            [-prediction.y, prediction.x, 0.0],
        ] );

        // let accel_dens = imu::ACCEL_NOISE *1e-6;
        // let rms= accel_dens * imu::ACCEL_BANDWIDTH.sqrt();

        // let accel_noise = Matrix3x3::new_diagonal(
        //     [rms * rms, rms * rms, rms * rms],
        // );

        const ACCEL_VAR: f32 = 0.01;
        let accel_noise = Matrix3x3::new_diagonal([ACCEL_VAR, ACCEL_VAR, ACCEL_VAR]);

        let s = h * self.orien_error_covariance * h.transpose() + accel_noise;
        let kalman_gain = self.orien_error_covariance * h.transpose() * s.inverse().map_err(KalmanFilterError::Matrix)?;

        let correction = kalman_gain * innovation;

        let n = libm::sqrtf(correction.x*correction.x + correction.y*correction.y + correction.z*correction.z);
        let max_corr = 0.7_f32;
        let correction = if n > max_corr {
            F32x3 { x: correction.x*max_corr/n, y: correction.y*max_corr/n, z: correction.z*max_corr/n }
        } else { correction };

        let dq = Self::normalize_exact(Quaternion::new(1.0, correction.x/2.0, correction.y/2.0, correction.z/2.0));

        self.orien_state_estimation = Self::normalize_exact(self.orien_state_estimation * dq);

        let i_kh = Matrix3x3::IDENTITY - kalman_gain * h;
        self.orien_error_covariance = i_kh * self.orien_error_covariance * i_kh.transpose() + kalman_gain * accel_noise * kalman_gain.transpose();

        Ok(())
    }

    async fn iir_filter(&mut self) -> Result<(), KalmanFilterError> {
        const ALPHA: f32 = 0.0;


        Ok(())
    }

    /// Returns the atitude state
    pub fn atitude(&self) -> Quaternion {
        self.orien_state_estimation
    }

    /// micromath's normalize function isn't very accurate, so using libm!!
    fn normalize_exact(q: Quaternion) -> Quaternion {
        let n = libm::sqrtf(q.norm());
        if n == 0.0 { return Quaternion::IDENTITY; }
        q.scale(1.0 / n)
    }

    pub async fn imu_dat(&mut self) {
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr).unwrap();
        let gyro = self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr).unwrap();


        info!("accel xyz: {} {} {}", accel.x, accel.y, accel.z);
        info!("gyro xyz: {} {} {}", gyro.x, gyro.y, gyro.z);

    }
}