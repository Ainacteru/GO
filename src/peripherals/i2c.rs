use crate::{self as go, Scl, Sda};
use atsamd_hal::{clock::GenericClockController, fugit::RateExtU32, pac::{self}};

pub struct I2c {
    pub i2c: go::I2c
}

impl I2c {
    pub fn new(pins: (impl Into<Sda>, impl Into<Scl>), sercom: go::I2cSercom, clocks: &mut GenericClockController, pm: &mut pac::Pm) -> Self {
        Self { 
            i2c: go::i2c_master(clocks, 100_u32.kHz(), sercom, pm, pins.0, pins.1) 
        }
    }
    pub fn inner(&mut self) -> &mut go::I2c {
        &mut self.i2c
    }
}