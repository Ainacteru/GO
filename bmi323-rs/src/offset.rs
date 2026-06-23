use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error, defs::*};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn get_accel_offset_gain(&mut self) -> Result<AccelOffsetGain, Error<E>> {
    self.read(Reg::AccDpOffX).await
  }

  pub async fn set_accel_offset_gain(&mut self, v: AccelOffsetGain) -> Result<(), Error<E>> {
    self.write(Reg::AccDpOffX, v).await
  }

  pub async fn get_gyro_offset_gain(&mut self) -> Result<GyroOffsetGain, Error<E>> {
    self.read(Reg::GyrDpOffX).await
  }

  pub async fn set_gyro_offset_gain(&mut self, v: GyroOffsetGain) -> Result<(), Error<E>> {
    self.write(Reg::GyrDpOffX, v).await
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 12)]
pub struct AccelOffsetGain {
  #[bits(14)]
  pub off_x: i16,
  #[skip(2)]
  #[bits(8)]
  pub gain_x: i16,
  #[skip(8)]
  #[bits(14)]
  pub off_y: i16,
  #[skip(2)]
  #[bits(8)]
  pub gain_y: i16,
  #[skip(8)]
  #[bits(14)]
  pub off_z: i16,
  #[skip(2)]
  #[bits(8)]
  pub gain_z: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 12)]
pub struct GyroOffsetGain {
  #[bits(10)]
  pub off_x: i16,
  #[skip(6)]
  #[bits(7)]
  pub gain_x: i16,
  #[skip(9)]
  #[bits(10)]
  pub off_y: i16,
  #[skip(6)]
  #[bits(7)]
  pub gain_y: i16,
  #[skip(9)]
  #[bits(10)]
  pub off_z: i16,
  #[skip(6)]
  #[bits(7)]
  pub gain_z: i16,
}
