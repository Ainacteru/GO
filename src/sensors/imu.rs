
use core::{f32, todo};

use atsamd_hal::{ehal::i2c::SevenBitAddress, ehal_async::{delay::DelayNs, i2c::I2c}};
use defmt::{debug, error};
use embassy_time::Instant;
use micromath::vector::F32x3;
use micromath::F32Ext;
use uom::si::{f32::ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

use crate::sensors::error::ImuError::{self};

const ADDRESS: u8 = 0x68;
const TAU: f32 = 0.5;

pub struct Imu<B, D> 
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs,
{
    i2c: B,
    delay: D,
    prev_time: Instant,
    prev_gyro: F32x3,
    prev_angle: F32x3
}

// driver impl block
impl<B, D> Imu<B, D>
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs,
{
    pub async fn new(i2c: B, delay: D) -> Result<Self, ImuError<>> {

        let mut imu = Self {
            i2c,
            delay,
            prev_time: Instant::now(),
            prev_gyro: F32x3 { x: 0.0, y: 0.0, z: 0.0 },
            prev_angle: F32x3 { x: 0.0, y: 0.0, z: 0.0 },
        };

        let addr_buf = imu.read( 0x00).await.map_err(|_| ImuError::I2C)?;
        let addr = addr_buf[0];

        match addr {
            0x43 => debug!("Found imu with addr {:#02X}", &addr),
            _ => error!("Imu address not matching 0x43, found: {:#02X}", &addr),
        }

        imu.soft_reset().await?;

        let err_buf = imu.read(0x01).await.map_err(|_| ImuError::I2C)?;
        let dev_status = err_buf[0];

        debug!("dev status {}", &dev_status);

        if dev_status != 0 {
            return Err(ImuError::Power);
        }

        let status_buf = imu.read(0x02).await.map_err(|_| ImuError::I2C)?;
        let sensor_status = status_buf[0];

        debug!("sens status {}", &sensor_status);

        if sensor_status & 1 == 0 {
            return Err(ImuError::Initialization);
        }

        imu.config_performance().await?;

        Ok(imu)
        
    }

    /// enums exist...
    async fn config_performance(&mut self) -> Result<(), ImuError> {
        let mode = 0x7000; //performance mode
        let averaging = 0x0; //no averaging
        let filter_odr = 0x0080; // odr / 4
        let odr = 0x0009; //200hz

        let accel_range = 0x0020; //8g
        let accel_config: u16 = mode | averaging | filter_odr | accel_range | odr; 

        self.write(0x20, accel_config).await.map_err(|_| ImuError::AccelConfig)?; // accel

        let gyro_range = 0x0040; //2000 d/s
        let gyro_config: u16 = mode | averaging | filter_odr | gyro_range | odr; 

        self.write(0x21, gyro_config).await.map_err(|_| ImuError::GyroConfig)?; // gyro   

        self.delay.delay_ms(50).await;
        Ok(())
    }
    pub async fn soft_reset(&mut self) -> Result<(), ImuError> {
        self.write(0x7E, 0xDEAF).await.map_err(|_| ImuError::SoftReset)?;

        self.delay.delay_ms(5).await;
        Ok(())
    }

    /// returns a vector 3d of the accel data in Gs
    pub async fn get_accel_data(&mut self) -> Result<F32x3, ImuError> {
        let mut buf = [0u8; 8];

        self.i2c.write_read(ADDRESS, &[0x03], &mut buf).await.map_err(|_| ImuError::AccelRead)?;

        let x = i16::from_le_bytes([buf[2], buf[3]]);
        let y = i16::from_le_bytes([buf[4], buf[5]]);
        let z = i16::from_le_bytes([buf[6], buf[7]]);

        const SCALE: f32 = 8.0 / 32768.0;

        Ok(F32x3 {
            x: -(x as f32 * SCALE),
            y: -(y as f32 * SCALE),
            z: z as f32 * SCALE,
        })
    }
    
    /// returns a vector 3d of the gyro data in degrees per second
    pub async fn get_gyro_data(&mut self) -> Result<F32x3, ImuError> {
        let mut buf = [0u8; 8];

        // not using the helper method here because i'm reading 3 registers at a time instead of 1
        self.i2c.write_read(ADDRESS, &[0x06], &mut buf).await.map_err(|_| ImuError::GyroRead)?;


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

    pub async fn get_temp_data(&mut self) -> Result<ThermodynamicTemperature, ImuError> {

        let temp = i16::from_le_bytes(self.read(0x09).await?);
        let temp  = temp as f32 / 512.0 + 23.0; 

        Ok(ThermodynamicTemperature::new::<degree_celsius>(temp))
    }

    /// `addr` is the address of the register
    /// 
    /// `data` is what you want to send as a u16
    async fn write(&mut self, addr: u8, data: u16) -> Result<(), ImuError>{
        let [lo, hi] = data.to_le_bytes();

        self.i2c.write(ADDRESS, &[addr, lo, hi]).await.map_err(|_| ImuError::I2C)?;

        Ok(())
    }

    /// `addr` is the address of the register
    /// 
    /// returns a 2 byte buffer without the 2 dummy bytes
    async fn read(&mut self, addr: u8) -> Result<[u8; 2], ImuError>{
        let mut buf = [0u8; 4];

        self.i2c.write_read(ADDRESS, &[addr], &mut buf).await.map_err(|_| ImuError::I2C)?;

        let buf: [u8; 2] = buf[2..=3].try_into().unwrap();

        Ok(buf)
    }
}

impl<B, D> Imu<B, D>
    where 
        B: I2c<SevenBitAddress>,
        D: DelayNs,
{
    pub async fn get_pitch_yaw_roll(&mut self) -> Result<F32x3, ImuError> {
       

        let accel = self.get_accel_data().await?;
        
        let a_pitch = (-accel.x).atan2((accel.y * accel.y + accel.z * accel.z).sqrt()) * 180.0 / f32::consts::PI;
        let a_roll = accel.y.atan2(accel.z) * 180.0 / f32::consts::PI;

        let gyro = self.get_gyro_data().await?;

        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_micros() as f32 / 1000000.0;
        let dt = dt.min(0.05);
        self.prev_time = now;

        let alpha = TAU / (TAU + dt);

        let x = alpha * (self.prev_angle.x + gyro.x * dt) + (1.0 - alpha) * a_pitch;
        let y = alpha * (self.prev_angle.y + gyro.y * dt) + (1.0 - alpha) * a_roll;
        let z = self.prev_angle.z + -(gyro.z * dt);

        self.prev_angle = F32x3 {x, y, z};
        

        Ok(self.prev_angle)
    }
}