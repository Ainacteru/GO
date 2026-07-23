use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use defmt::{debug, error, info};
use embassy_time::{Delay};
use micromath::vector::{Component, F32x3, Vector3d};

use crate::sensors::error::ImuError;

const ADDRESS: u8 = 0x68;

pub struct Imu<B, D> 
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs,
{
    i2c: B,
    delay: D,
}

impl<B, D> Imu<B, D>
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs,
{
    pub async fn new(mut i2c: B, delay: D) -> Result<Self, ImuError> {


        let mut addr_buf = [0u8; 3]; // 2 dummy bytes
        i2c.write_read(ADDRESS, &[0x00], &mut addr_buf).await.unwrap();
        let addr = addr_buf[2];

        match addr {
            0x43 => info!("Found imu with addr {:#02X}", &addr),
            _ => error!("Imu address not matching 0x43, found: {:#02X}", &addr),
        }

        let mut err_buf = [0u8; 3];
        i2c.write_read(ADDRESS, &[0x01], &mut err_buf).await.unwrap();
        let dev_status = err_buf[2];

        debug!("dev status {:#02X}", &dev_status);

        if dev_status != 0 {
            return Err(ImuError::Power);
        }

        let mut status_buf = [0u8; 3];
        i2c.write_read(ADDRESS, &[0x02], &mut status_buf).await.unwrap();
        let sensor_status = status_buf[2];

        debug!("sens status {:#02X}", &sensor_status);

        if sensor_status != 1 {
            return Err(ImuError::Initialization);
        }

        let mut imu = Self {
            i2c,
            delay,
        };

        imu.config_performance().await;

        Ok(imu)
        
    }

    /// enums exist...
    pub async fn config_performance(&mut self) {
        let i2c = &mut self.i2c;

        let mode = 0x7000; //performance mode
        let averaging = 0x0; //no averaging
        let filter_odr = 0x0080; // odr / 4
        let odr = 0x0009; //200hz

        let accel_range = 0x0020; //8g
        let accel_config: u16 = mode | averaging | filter_odr | accel_range | odr; 
        let [accel_lo, accel_hi] = accel_config.to_le_bytes();

        i2c.write(ADDRESS, &[0x20, accel_lo, accel_hi]).await.unwrap(); // accel

        let gyro_range = 0x0040; //2kdps
        let gyro_config: u16 = mode | averaging | filter_odr | gyro_range | odr; 
        let [gyro_lo, gyro_hi] = gyro_config.to_le_bytes();

        i2c.write(ADDRESS, &[0x21, gyro_lo, gyro_hi]).await.unwrap(); // gyro
    }

    pub async fn get_accel_data(&mut self) -> Result<F32x3, ImuError> {
        let mut buf = [0u8; 8];

        let read = self.i2c.write_read(ADDRESS, &[0x03], &mut buf).await;

        if read.is_err() {
            return Err(ImuError::AccelRead)
        }

        let x = i16::from_le_bytes([buf[2], buf[3]]);
        let y = i16::from_le_bytes([buf[4], buf[5]]);
        let z = i16::from_le_bytes([buf[6], buf[7]]);

        const SCALE: f32 = 8.0 / 32768.0;

        Ok(F32x3 {
            x: x as f32 * SCALE,
            y: y as f32 * SCALE,
            z: z as f32 * SCALE,
        })
    }
    
    pub async fn get_gyro_data(&mut self) -> Result<F32x3, ImuError> {
        let mut buf = [0u8; 8];

        let read = self.i2c.write_read(ADDRESS, &[0x06], &mut buf).await;

        if read.is_err() {
            return Err(ImuError::GyroRead)
        }

        let x = i16::from_le_bytes([buf[2], buf[3]]);
        let y = i16::from_le_bytes([buf[4], buf[5]]);
        let z = i16::from_le_bytes([buf[6], buf[7]]);

        const SCALE: f32 = 2000.0 / 32768.0;

        Ok(F32x3 {
            x: x as f32 * SCALE,
            y: y as f32 * SCALE,
            z: z as f32 * SCALE,
        })
    }
}