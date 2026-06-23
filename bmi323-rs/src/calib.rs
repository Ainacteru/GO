use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Gyro Smart Calibration: write raw select block (bytes, little‑endian words).
  pub async fn set_gyro_sc_select(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
    self.write_feature_bytes(super::FeatureAddr::GyroScSelect, bytes).await
  }

  /// Gyro Smart Calibration: read raw select block (bytes, little‑endian words).
  pub async fn get_gyro_sc_select(&mut self, out: &mut [u8]) -> Result<(), Error<E>> {
    self.read_feature_bytes(super::FeatureAddr::GyroScSelect, out).await
  }

  /// Gyro Smart Calibration: write raw ST configuration block (bytes).
  pub async fn set_gyro_sc_st_conf(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
    self.write_feature_bytes(super::FeatureAddr::GyroScStConf, bytes).await
  }

  /// Gyro Smart Calibration: read raw ST configuration block (bytes).
  pub async fn get_gyro_sc_st_conf(&mut self, out: &mut [u8]) -> Result<(), Error<E>> {
    self.read_feature_bytes(super::FeatureAddr::GyroScStConf, out).await
  }

  /// Gyro Smart Calibration: read raw coefficients block (bytes).
  pub async fn get_gyro_sc_coefficients(&mut self, out: &mut [u8]) -> Result<(), Error<E>> {
    self
      .read_feature_bytes(super::FeatureAddr::GyroScStCoefficients, out)
      .await
  }

  /// Alternate auto-config: write raw block (bytes).
  pub async fn set_alt_auto_config(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
    self.write_feature_bytes(super::FeatureAddr::AltAutoConfig, bytes).await
  }

  /// Alternate auto-config: read raw block (bytes).
  pub async fn get_alt_auto_config(&mut self, out: &mut [u8]) -> Result<(), Error<E>> {
    self.read_feature_bytes(super::FeatureAddr::AltAutoConfig, out).await
  }
}
