//! Common types used across the BMI323 driver.

/// 3-axis sensor data (raw 16-bit values).
///
/// Used for reading raw accelerometer and gyroscope data directly
/// from the sensor registers.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 6)]
pub struct XYZ {
  /// X-axis raw value
  #[bits(16)]
  pub x: i16,
  /// Y-axis raw value
  #[bits(16)]
  pub y: i16,
  /// Z-axis raw value
  #[bits(16)]
  pub z: i16,
}

/// Output data rate (ODR) for accelerometer and gyroscope.
///
/// Higher ODR values reduce latency but increase power consumption
/// and data bandwidth requirements. Choose based on your application's
/// sampling rate needs.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputDataRate {
  Hz0_78 = 0x01,
  Hz1_56 = 0x02,
  Hz3_12 = 0x03,
  Hz6_25 = 0x04,
  Hz12_5 = 0x05,
  Hz25 = 0x06,
  Hz50 = 0x07,
  Hz100 = 0x08,
  Hz200 = 0x09,
  Hz400 = 0x0A,
  Hz800 = 0x0B,
  Hz1600 = 0x0C,
  Hz3200 = 0x0D,
  Hz6400 = 0x0E,
}

impl From<OutputDataRate> for u8 {
  fn from(odr: OutputDataRate) -> Self {
    odr as u8
  }
}

impl TryFrom<u8> for OutputDataRate {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x01 => Ok(OutputDataRate::Hz0_78),
      0x02 => Ok(OutputDataRate::Hz1_56),
      0x03 => Ok(OutputDataRate::Hz3_12),
      0x04 => Ok(OutputDataRate::Hz6_25),
      0x05 => Ok(OutputDataRate::Hz12_5),
      0x06 => Ok(OutputDataRate::Hz25),
      0x07 => Ok(OutputDataRate::Hz50),
      0x08 => Ok(OutputDataRate::Hz100),
      0x09 => Ok(OutputDataRate::Hz200),
      0x0A => Ok(OutputDataRate::Hz400),
      0x0B => Ok(OutputDataRate::Hz800),
      0x0C => Ok(OutputDataRate::Hz1600),
      0x0D => Ok(OutputDataRate::Hz3200),
      0x0E => Ok(OutputDataRate::Hz6400),
      _ => Err(()),
    }
  }
}

/// Digital low‑pass bandwidth selection. The cutoff is relative to ODR.
/// - `OdrHalf`: fc ≈ ODR/2 (wider bandwidth)
/// - `OdrQuarter`: fc ≈ ODR/4 (narrower bandwidth)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Bandwidth {
  OdrHalf = 0x00,
  OdrQuarter = 0x01,
}

impl From<Bandwidth> for u8 {
  fn from(value: Bandwidth) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for Bandwidth {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(Bandwidth::OdrHalf),
      0x01 => Ok(Bandwidth::OdrQuarter),
      _ => Err(()),
    }
  }
}

/// Number of samples to average. Reduces noise at the cost of latency and
/// responsiveness. Applies to both data and some feature engines.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AverageNum {
  /// No averaging
  No = 0x00,
  /// Average 2 samples
  Avg2 = 0x01,
  /// Average 4 samples
  Avg4 = 0x02,
  /// Average 8 samples
  Avg8 = 0x03,
  /// Average 16 samples
  Avg16 = 0x04,
  /// Average 32 samples
  Avg32 = 0x05,
  /// Average 64 samples
  Avg64 = 0x06,
}

impl From<AverageNum> for u8 {
  fn from(value: AverageNum) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for AverageNum {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0x00 => Ok(AverageNum::No),
      0x01 => Ok(AverageNum::Avg2),
      0x02 => Ok(AverageNum::Avg4),
      0x03 => Ok(AverageNum::Avg8),
      0x04 => Ok(AverageNum::Avg16),
      0x05 => Ok(AverageNum::Avg32),
      0x06 => Ok(AverageNum::Avg64),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Sensor {
  Accel,
  Gyro,
  Temp,
}
