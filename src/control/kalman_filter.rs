use core::f32;

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use embassy_time::Instant;

use crate::{control::error::KalmanFilterError, sensors::imu::{self, Imu}, util::math::matrix::{Matrix, Matrix3x3}};
use micromath::{F32Ext, Quaternion, vector::{F32x3, Vector}};

pub struct KalmanFilter <B: I2c<SevenBitAddress>, D: DelayNs> {
    imu: Imu<B, D>,
    prev_time: Instant,
    state_estimation: Quaternion,
    error_covariance: Matrix3x3,
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {

    pub fn new(imu: Imu<B, D>) -> Self {
        Self {
            imu,
            prev_time: Instant::now(),
            state_estimation: Quaternion::IDENTITY,
            error_covariance: Matrix3x3::new_diagonal([0.01, 0.01, 0.01])
        }
    }

    pub async fn predict(&mut self) -> Result<(), KalmanFilterError> {
        const DEG2RAD: f32 = f32::consts::PI / 180.0;
        let gyro =  self.imu.get_gyro_data().await.map_err(KalmanFilterError::ImuErr)?;

        let w = F32x3 {
          x: gyro.x * DEG2RAD,
          y: gyro.y * DEG2RAD,
          z: gyro.z * DEG2RAD,
        };

        let omega = Quaternion::new(0.0, w.x, w.y, w.z);

        let q_dot = 0.5 * self.state_estimation * omega; // f(x, u)

        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_micros() as f32 / 1000000.0;
        let dt = dt.min(0.05);
        self.prev_time = now;

        self.state_estimation = Self::normalize_exact(self.state_estimation + q_dot * dt);

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
        let sigma = imu::GYRO_NOISE * imu::GYRO_BANDWIDTH.sqrt() * f32::consts::PI / 180.0;
        let q_value = sigma * sigma * dt;

        let q_noise = Matrix3x3::new_diagonal(
            [q_value, q_value, q_value]
        );

        // P = FPF^T + Q
        self.error_covariance = f * self.error_covariance * f.transpose() + q_noise;

        Ok(())
    }

    async fn correct(&mut self) -> Result<(), KalmanFilterError> {
        let accel = self.imu.get_accel_data().await.map_err(KalmanFilterError::ImuErr)?;

        let mag = libm::sqrtf(accel.x*accel.x + accel.y*accel.y + accel.z*accel.z);

        if !(0.9..=1.1).contains(&mag) {
            return Ok(())
        }

        let accel = F32x3 { x: accel.x / mag, y: accel.y / mag, z: accel.z / mag };

        let gravity = F32x3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };

        // R(q)
        let prediction = self.state_estimation.conj().rotate(gravity);

        let innovation = accel - prediction;

        let h = Matrix3x3::from_array( [
            [0.0, -prediction.z, prediction.y],
            [prediction.z, 0.0, -prediction.x],
            [-prediction.y, prediction.x, 0.0],
        ] );

        let accel_dens = imu::ACCEL_NOISE *1e-6;
        let rms= accel_dens * imu::ACCEL_BANDWIDTH.sqrt();

        let accel_noise = Matrix3x3::new_diagonal(
            [rms * rms, rms * rms, rms * rms],
        );

        let s = h * self.error_covariance * h.transpose() + accel_noise;
        let kalman_gain = self.error_covariance * h.transpose() * s.inverse().map_err(KalmanFilterError::Matrix)?;

        let correction = kalman_gain.multiply_vector(innovation);

        let dq = Self::normalize_exact(Quaternion::new(1.0, correction.x/2.0, correction.y/2.0, correction.z/2.0));

        self.state_estimation = Self::normalize_exact(dq * self.state_estimation);

        let i_kh = Matrix3x3::IDENTITY - kalman_gain * h;
        self.error_covariance = i_kh * self.error_covariance * i_kh.transpose() + kalman_gain * accel_noise * kalman_gain.transpose();

        Ok(())
    }

    pub async fn filter(&mut self) -> Result<(), KalmanFilterError> {

        self.predict().await?;

        self.correct().await?;

        Ok(())
    }

    pub fn state(&self) -> Quaternion {
        self.state_estimation
    }

    /// micromath's normalize function isn't very accurate, so using libm!!
    fn normalize_exact(q: Quaternion) -> Quaternion {
        let n = libm::sqrtf(q.norm());
        if n == 0.0 { return Quaternion::IDENTITY; }
        q.scale(1.0 / n)
    }
}