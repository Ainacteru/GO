use atsamd_hal::{ehal::i2c::I2c as I2cTrait,};
use crate::peripherals::i2c::I2c;

const ADDRESS: u8 = 0x77;

pub struct Bmp {
    i2c: I2c,
    t_lin: f32,
    par_t1: f32,
    par_t2: f32,
    par_t3: f32,
}

impl Bmp {
    pub fn new(mut i2c: I2c) -> Self {
        let mut calib = [0u8; 21];
        i2c.inner().write_read(ADDRESS, &[0x31], &mut calib).unwrap();

        Self {
            i2c,
            t_lin: 0.0,
            par_t1: u16::from_le_bytes([calib[0], calib[1]]) as f32 / 0.00390625, // / 2^-8
            par_t2: u16::from_le_bytes([calib[2], calib[3]]) as f32 / 1073741824.0, // / 2^30
            par_t3: calib[4] as i8 as f32 / 281474976710656.0, // / 2^48
        }
    }

    pub fn read_temperature(&mut self) -> f32 {
        let mut buf = [0u8; 3];
        self.i2c.inner().write_read(ADDRESS, &[0x07], &mut buf).unwrap();
        let raw = (buf[2] as u32) << 16 | (buf[1] as u32) << 8 | buf[0] as u32;

        let partial1 = raw as f32 - self.par_t1;
        let partial2 = partial1 * self.par_t2;
        self.t_lin = partial2 + (partial1 * partial1) * self.par_t3;
        self.t_lin
    }
}