use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error, defs::*};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Get/set I2C interface control register (raw).
  pub async fn get_i2c_if(&mut self) -> Result<u8, Error<E>> {
    self.read_u8(Reg::IoI2cIf).await
  }

  pub async fn set_i2c_if(&mut self, v: u8) -> Result<(), Error<E>> {
    self.write_bytes(Reg::IoI2cIf, &[v]).await
  }

  /// Get/set SPI interface control register (raw).
  pub async fn get_spi_if(&mut self) -> Result<u8, Error<E>> {
    self.read_u8(Reg::IoSpiIf).await
  }

  pub async fn set_spi_if(&mut self, v: u8) -> Result<(), Error<E>> {
    self.write_bytes(Reg::IoSpiIf, &[v]).await
  }

  /// Get/set IO pad drive strength (raw).
  pub async fn get_io_pad_strength(&mut self) -> Result<u8, Error<E>> {
    self.read_u8(Reg::IoPadStrength).await
  }

  pub async fn set_io_pad_strength(&mut self, v: u8) -> Result<(), Error<E>> {
    self.write_bytes(Reg::IoPadStrength, &[v]).await
  }

  /// Get/set ODR deviation (raw).
  pub async fn get_io_odr_deviation(&mut self) -> Result<u8, Error<E>> {
    self.read_u8(Reg::IoOdrDeviation).await
  }

  pub async fn set_io_odr_deviation(&mut self, v: u8) -> Result<(), Error<E>> {
    self.write_bytes(Reg::IoOdrDeviation, &[v]).await
  }

  /// Get/set IO power-down control (raw).
  pub async fn get_io_pdn_ctrl(&mut self) -> Result<u8, Error<E>> {
    self.read_u8(Reg::IoPdnCtrl).await
  }

  pub async fn set_io_pdn_ctrl(&mut self, v: u8) -> Result<(), Error<E>> {
    self.write_bytes(Reg::IoPdnCtrl, &[v]).await
  }
}
