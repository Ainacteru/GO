use core::cell::RefCell;

use atsamd_hal::{ehal::i2c::I2c as I2cTrait};
use crate::{peripherals::i2c::I2c, sensors::errors::SensorError::{self, NoId}};

const ADDRESS: u8 = 0x68;
pub struct Imu<'a> {
    i2c: &'a RefCell<I2c>
}

impl<'a> Imu<'a> {
    pub fn new(i2c: &'a RefCell<I2c>) -> Self {
        Self {
            i2c
        }
    }
    pub fn init(&mut self) {
        self.get_id().unwrap();
        
    }

    pub fn get_id(&mut self) -> Result<u8, SensorError> {
        let mut buf = [0u8; 4]; // 2 dummy + 2 data bytes
        match self.i2c.borrow_mut().inner().write_read(ADDRESS, &[0x00], &mut buf) {
            Ok(_) => Ok(buf[2]),
            Err(_) => Err(NoId),
        }
    }
}