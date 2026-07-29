use core::f32;

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use embassy_time::Instant;

use crate::{control::error::KalmanFilterError, sensors::imu::Imu};
use micromath::{F32Ext, Quaternion, vector::F32x2};

pub struct KalmanFilter <B: I2c<SevenBitAddress>, D: DelayNs> {
    imu: Imu<B, D>,
    pub state_estimation: Quaternion,
    prev_time: Instant,
}

impl <B: I2c<SevenBitAddress>, D: DelayNs> KalmanFilter <B, D> {

    pub fn new(imu: Imu<B, D>) -> Self {
        Self {
            imu,
            state_estimation: Quaternion::IDENTITY,
            prev_time: Instant::now(),
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

        Ok(())
    }

}