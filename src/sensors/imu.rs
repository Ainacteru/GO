use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::i2c::I2c};
use bmi323::{Bmi323, accel::AccelConfig, gyro::GyroConfig};
use defmt::info;
use embassy_time::{Delay, Timer};
use micromath::vector::Vector3d;

pub struct Imu<B: I2c<SevenBitAddress>> {
    bmi: Bmi323<B, Delay, ()>
}

impl<B: I2c<SevenBitAddress>> Imu<B> {
    pub async fn new(i2c: B, delay: Delay) -> Self {
        let mut bmi = Bmi323::new(i2c, delay);
        bmi.soft_reset().await.unwrap();

        let id = bmi.get_id().await.unwrap();

        match id {
            0x43 => info!("Found BMI323 with id {:#x}", id),
            _ => info!("Found device with id {:#x} while searching for BMI323 with id 0x43", id),
        }

        Timer::after_millis(5).await;

        let accel_config = AccelConfig::default();
        bmi.set_accel_conf(accel_config).await.unwrap();
        
        let gyro_config = GyroConfig::default();
        bmi.set_gyro_conf(gyro_config).await.unwrap();

        

        Self {
            bmi
        }
    }

    pub async fn get_accel_data(&mut self) -> Vector3d<f32>{
        self.bmi.get_accel_data().await.unwrap()
    }
    
    pub async fn get_gyro_data(&mut self) -> Vector3d<f32>{
        self.bmi.get_gyro_data().await.unwrap()
    }
}