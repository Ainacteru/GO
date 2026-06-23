use embedded_hal_async::{delay::DelayNs, i2c::*};

use super::{Bmi323, Error, defs::*};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Configure alternate config switching behavior (accel/gyro, reset-on-user-write).
  pub async fn set_alt_enable(&mut self, en: AltEnable) -> Result<(), Error<E>> {
    self.write(Reg::AltConf, en).await
  }

  /// Program alternate accelerometer configuration (does not activate by itself).
  pub async fn set_alt_accel_conf(&mut self, cfg: crate::accel::AccelConfig) -> Result<(), Error<E>> {
    self.write(Reg::AltAccConf, cfg).await
  }

  /// Program alternate gyroscope configuration (does not activate by itself).
  pub async fn set_alt_gyro_conf(&mut self, cfg: crate::gyro::GyroConfig) -> Result<(), Error<E>> {
    self.write(Reg::AltGyrConf, cfg).await
  }

  /// Read active sources for accel/gyro configs. Returns (accel_uses_alt, gyro_uses_alt).
  pub async fn get_alt_status(&mut self) -> Result<AltStatus, Error<E>> {
    self.read(Reg::AltStatus).await
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct AltEnable {
  #[bits(1)]
  pub accel: bool,
  #[skip(3)]
  #[bits(1)]
  pub gyro: bool,
  /// If enabled, writes to ACC_CONF/GYR_CONF reset active alt to user config.
  ///
  /// Context:
  /// - Alternate configs are useful for low‑power/background vs active modes.
  ///   Feature events or host policy can switch between user and alternate.
  /// - `reset_on_user_conf_write=true` forces a swap back to the user copy when
  ///   you touch ACC_CONF/GYR_CONF, which simplifies reverting to baseline.
  #[skip(3)]
  #[bits(1)]
  pub reset_on_user_conf_write: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct AltStatus {
  #[bits(1)]
  pub accel: bool,
  #[skip(3)]
  #[bits(1)]
  pub gyro: bool,
}
