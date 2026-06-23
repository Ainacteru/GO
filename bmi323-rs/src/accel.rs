//! Accelerometer configuration and data reading.
//!
//! This module provides functions to configure the BMI323 accelerometer
//! and read acceleration data.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
//! use bmi323::accel::{AccelConfig, AccelRange};
//! use bmi323::OutputDataRate;
//!
//! // Configure with default settings
//! imu.set_accel_conf(AccelConfig::default()).await.unwrap();
//!
//! // Or customize configuration
//! let config = AccelConfig {
//!     odr: OutputDataRate::Hz100,
//!     range: AccelRange::G4,
//!     ..Default::default()
//! };
//! imu.set_accel_conf(config).await.unwrap();
//!
//! // Read acceleration data in g units
//! let accel = imu.get_accel_data().await.unwrap();
//! println!("Accel: x={}, y={}, z={}", accel.x, accel.y, accel.z);
//! # }
//! ```

use embedded_hal_async::{delay::DelayNs, i2c::*};
use micromath::vector::Vector3d;

use super::{defs::*, Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn get_accel_conf(&mut self) -> Result<AccelConfig, Error<E>> {
    self.read(Reg::AccConf).await
  }

  pub async fn set_accel_conf(&mut self, cfg: AccelConfig) -> Result<(), Error<E>> {
    self.accel_mult = cfg.range.multiplier();
    self.write(Reg::AccConf, cfg).await?;
    self.wait_for(crate::Sensor::Accel).await
  }

  /// Read raw accelerometer data (16-bit signed integers).
  ///
  /// Returns raw ADC values. Use [`get_accel_data`](Self::get_accel_data)
  /// to get scaled values in g units.
  pub async fn get_raw_accel_data(&mut self) -> Result<Vector3d<i16>, Error<E>> {
    let mut b = [0u8; 6];
    self.read_bytes(Reg::AccDataX, &mut b).await?;
    Ok(Vector3d {
      x: i16::from_le_bytes([b[0], b[1]]),
      y: i16::from_le_bytes([b[2], b[3]]),
      z: i16::from_le_bytes([b[4], b[5]]),
    })
  }

  /// Read accelerometer data scaled to g units.
  ///
  /// Returns acceleration in g (standard gravity, 9.81 m/s²) for each axis.
  /// The scaling is automatically applied based on the configured range.
  pub async fn get_accel_data(&mut self) -> Result<Vector3d<f32>, Error<E>> {
    let accel_data = self.get_raw_accel_data().await?;
    let range = self.accel_mult;

    Ok(Vector3d { x: accel_data.x as f32 * range, y: accel_data.y as f32 * range, z: accel_data.z as f32 * range })
  }
}

/// Accelerometer configuration register.
///
/// Configure the accelerometer's output data rate, measurement range,
/// bandwidth, averaging, and power mode.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct AccelConfig {
  /// Output Data Rate in Hz (≈0.78 Hz to 6.4 kHz).
  #[bits(4)]
  pub odr: crate::OutputDataRate,
  /// Full-scale measurement range (±2g, ±4g, ±8g, or ±16g).
  #[bits(3)]
  pub range: AccelRange,
  /// Digital low-pass filter cutoff (ODR/2 or ODR/4).
  #[bits(1)]
  pub bw: crate::Bandwidth,
  /// Number of samples averaged by on-chip filter.
  #[bits(3)]
  pub avg: crate::AverageNum,
  #[skip(1)]
  /// Power mode (normal, low power, etc.).
  #[bits(3)]
  pub mode: AccelPowerMode,
}

impl Default for AccelConfig {
  fn default() -> Self {
    AccelConfig {
      odr: crate::OutputDataRate::Hz1600,
      range: AccelRange::G2,
      bw: crate::Bandwidth::OdrHalf,
      avg: crate::AverageNum::No,
      mode: AccelPowerMode::Normal,
    }
  }
}

/// Accelerometer measurement range.
///
/// Determines the full-scale range of acceleration measurements.
/// Higher ranges allow measuring stronger accelerations but with
/// lower resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AccelRange {
  /// ±2g range
  G2 = 0x00,
  /// ±4g range
  G4 = 0x01,
  /// ±8g range
  G8 = 0x02,
  /// ±16g range
  G16 = 0x03,
}

impl AccelRange {
  pub(crate) fn multiplier(self) -> f32 {
    match self {
      AccelRange::G2 => 1. / 16384.,
      AccelRange::G4 => 1. / 8192.,
      AccelRange::G8 => 1. / 4096.,
      AccelRange::G16 => 1. / 2048.,
    }
  }
}

impl From<AccelRange> for u8 {
  fn from(value: AccelRange) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for AccelRange {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(AccelRange::G2),
      0x01 => Ok(AccelRange::G4),
      0x02 => Ok(AccelRange::G8),
      0x03 => Ok(AccelRange::G16),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AccelPowerMode {
  Disable = 0x0,
  LowPower = 0x3,
  Normal = 0x4,
  HighPerf = 0x7,
}

impl From<AccelPowerMode> for u8 {
  fn from(value: AccelPowerMode) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for AccelPowerMode {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x0 => Ok(AccelPowerMode::Disable),
      0x3 => Ok(AccelPowerMode::LowPower),
      0x4 => Ok(AccelPowerMode::Normal),
      0x7 => Ok(AccelPowerMode::HighPerf),
      _ => Err(()),
    }
  }
}
