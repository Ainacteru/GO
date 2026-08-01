use core::f32;

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use embassy_time::Instant;

use crate::{control::error::KalmanFilterError, sensors::imu::{self, Imu}, util::math::matrix::{Matrix, Matrix3x3}};
use micromath::{F32Ext, Quaternion, vector::F32x2};

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

        let omega = Quaternion::new(0.0, gyro.x * DEG2RAD, gyro.y * DEG2RAD, gyro.z * DEG2RAD);

        let q_dot = 0.5 * self.state_estimation * omega; // f(x, u)

        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_micros() as f32 / 1000000.0;
        let dt = dt.min(0.05);
        self.prev_time = now;

        self.state_estimation += q_dot * dt;
        self.state_estimation = self.state_estimation.normalize();

        // error covariance matrix update
        

        let mut skew = Matrix3x3::from_array([
            [0.0, -gyro.z, gyro.y],
            [gyro.z, 0.0, -gyro.x],
            [-gyro.y, gyro.x, 0.0],
        ]);

        // F = I - skew * dt
        skew.scale(dt);
        let mut f = Matrix3x3::IDENTITY - skew;
        let f_transpose = f.transpose();
        // transpose F
        // Q
        // sec/hz to rads/sec
        let sigma = imu::GYRO_NOISE * imu::GYRO_BANDWIDTH.sqrt() * f32::consts::PI / 180.0;
        let q_value = sigma * sigma * dt * dt;

        let q_noise = Matrix3x3::from_array([
            [q_value, 0.0, 0.0],
            [0.0, q_value, 0.0],
            [0.0, 0.0, q_value],
        ]);

        // P = FPF^T + Q
        self.error_covariance = f * self.error_covariance * f_transpose + q_noise;

        Ok(())
    }

    pub fn state(&self) -> Quaternion {
        self.state_estimation
    }
}