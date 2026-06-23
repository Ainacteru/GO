//! Gyroscope configuration and data reading.
//!
//! This module provides functions to configure the BMI323 gyroscope
//! and read angular velocity data.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
//! use bmi323::gyro::{GyroConfig, GyroRange};
//! use bmi323::OutputDataRate;
//!
//! // Configure with default settings
//! imu.set_gyro_conf(GyroConfig::default()).await.unwrap();
//!
//! // Or customize configuration
//! let config = GyroConfig {
//!     odr: OutputDataRate::Hz100,
//!     range: GyroRange::DPS500,
//!     ..Default::default()
//! };
//! imu.set_gyro_conf(config).await.unwrap();
//!
//! // Read gyroscope data in degrees per second
//! let gyro = imu.get_gyro_data().await.unwrap();
//! println!("Gyro: x={}, y={}, z={}", gyro.x, gyro.y, gyro.z);
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
  /// Read the current gyroscope configuration.
  pub async fn get_gyro_conf(&mut self) -> Result<GyroConfig, Error<E>> {
    self.read(Reg::GyrConf).await
  }

  /// Set the gyroscope configuration.
  ///
  /// This configures the output data rate, measurement range, bandwidth,
  /// averaging, and power mode. The function waits for the gyroscope
  /// to be ready after applying the configuration.
  pub async fn set_gyro_conf(&mut self, cfg: GyroConfig) -> Result<(), Error<E>> {
    self.gyro_mult = cfg.range.multiplier();
    self.write(Reg::GyrConf, cfg).await?;
    // Fixed delay for gyro startup instead of polling drdy bit
    self.delay.delay_ms(100).await;
    Ok(())
  }

  /// Read raw gyroscope data (16-bit signed integers).
  ///
  /// Returns raw ADC values. Use [`get_gyro_data`](Self::get_gyro_data)
  /// to get scaled values in degrees per second.
  pub async fn get_raw_gyro_data(&mut self) -> Result<Vector3d<i16>, Error<E>> {
    let mut b = [0u8; 6];
    self.read_bytes(Reg::GyrDataX, &mut b).await?;
    Ok(Vector3d {
      x: i16::from_le_bytes([b[0], b[1]]),
      y: i16::from_le_bytes([b[2], b[3]]),
      z: i16::from_le_bytes([b[4], b[5]]),
    })
  }

  /// Read gyroscope data scaled to degrees per second (°/s).
  ///
  /// Returns angular velocity in degrees per second for each axis.
  /// The scaling is automatically applied based on the configured range.
  pub async fn get_gyro_data(&mut self) -> Result<Vector3d<f32>, Error<E>> {
    let accel_data = self.get_raw_gyro_data().await?;
    let range = self.gyro_mult;

    Ok(Vector3d { x: accel_data.x as f32 * range, y: accel_data.y as f32 * range, z: accel_data.z as f32 * range })
  }
}

/// Gyroscope configuration register.
///
/// Configure the gyroscope's output data rate, measurement range,
/// bandwidth, averaging, and power mode.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct GyroConfig {
  /// Output Data Rate in Hz (≈0.78 Hz to 6.4 kHz).
  #[bits(4)]
  pub odr: crate::OutputDataRate,
  /// Full-scale measurement range (±125°/s to ±2000°/s).
  #[bits(3)]
  pub range: GyroRange,
  /// Digital low-pass filter cutoff (ODR/2 or ODR/4).
  #[bits(1)]
  pub bw: crate::Bandwidth,
  /// Number of samples averaged by on-chip filter.
  #[bits(3)]
  pub avg: crate::AverageNum,
  #[skip(1)]
  /// Power mode (normal, low power, etc.).
  #[bits(3)]
  pub mode: GyroPowerMode,
}

impl Default for GyroConfig {
  fn default() -> Self {
    GyroConfig {
      odr: crate::OutputDataRate::Hz100,
      range: GyroRange::DPS2000,
      bw: crate::Bandwidth::OdrQuarter,
      avg: crate::AverageNum::No,
      mode: GyroPowerMode::Normal,
    }
  }
}

/// Gyroscope measurement ranges
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GyroRange {
  /// ±125 degrees per second
  DPS125 = 0,
  /// ±250 degrees per second
  DPS250 = 1,
  /// ±500 degrees per second
  DPS500 = 2,
  /// ±1000 degrees per second
  DPS1000 = 3,
  /// ±2000 degrees per second
  DPS2000 = 4,
}

impl GyroRange {
  pub fn dps(&self) -> f32 {
    match self {
      GyroRange::DPS125 => 125.0,
      GyroRange::DPS250 => 250.0,
      GyroRange::DPS500 => 500.0,
      GyroRange::DPS1000 => 1000.0,
      GyroRange::DPS2000 => 2000.0,
    }
  }

  pub fn multiplier(&self) -> f32 {
    self.dps() / f32::from(i16::MAX)
  }
}

impl From<GyroRange> for u8 {
  fn from(value: GyroRange) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for GyroRange {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(GyroRange::DPS125),
      1 => Ok(GyroRange::DPS250),
      2 => Ok(GyroRange::DPS500),
      3 => Ok(GyroRange::DPS1000),
      4 => Ok(GyroRange::DPS2000),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GyroPowerMode {
  /// Gyroscope disabled
  Disable = 0x00,
  /// Supend mode
  Suspend = 0x01,
  /// Low power mode
  LowPower = 0x03,
  /// Normal power mode
  Normal = 0x04,
  /// High perfomance mode
  HighPerf = 0x07,
}

impl From<GyroPowerMode> for u8 {
  fn from(value: GyroPowerMode) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for GyroPowerMode {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(GyroPowerMode::Disable),
      0x01 => Ok(GyroPowerMode::Suspend),
      0x03 => Ok(GyroPowerMode::LowPower),
      0x04 => Ok(GyroPowerMode::Normal),
      0x07 => Ok(GyroPowerMode::HighPerf),
      _ => Err(()),
    }
  }
}
