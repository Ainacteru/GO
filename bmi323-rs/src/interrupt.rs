//! Interrupt configuration and status reading.
//!
//! The BMI323 has two configurable interrupt pins (INT1 and INT2) that can
//! be mapped to various sensor events and features.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
//! use bmi323::interrupt::*;
//!
//! // Configure INT1 as active-high push-pull
//! let int1 = IntConfig {
//!     level: IntLevel::ActiveHigh,
//!     output: IntOutputMode::PushPull,
//!     enable: true,
//! };
//! imu.set_int_config(true, int1, IntConfig::default()).await.unwrap();
//!
//! // Map features to interrupt pins
//! let int_map = IntMap {
//!     any_motion: IntPin::Int1,
//!     tap: IntPin::Int1,
//!     ..Default::default()
//! };
//! imu.set_int_map(int_map).await.unwrap();
//!
//! // Read interrupt status
//! let status = imu.get_int1_status().await.unwrap();
//! # }
//! ```

use embedded_hal_async::{delay::DelayNs, i2c::*};

use super::{defs::*, Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_int_map(&mut self, map: IntMap) -> Result<(), Error<E>> {
    self.write(Reg::IntMap1, map).await
  }

  /// Set the interrupt latch mode (mapped to `INT_CONF.latch`).
  ///
  /// Official behavior:
  /// - Non‑latch: status bits clear automatically once the condition is gone.
  /// - Latch: status bits remain set until explicitly cleared (e.g., by reading
  ///   the status register), enabling edge‑style interrupt handling.
  pub async fn set_int_latch(&mut self, latch: bool) -> Result<(), Error<E>> {
    // Read‑modify‑write the latch bit in INT_CONF using bitfields
    let mut r: IntConfReg = self.read(Reg::IntConf).await?;
    r.latch = latch;
    self.write(Reg::IntConf, r).await
  }

  pub async fn set_int_pins(&mut self, pin1: IntConfig, pin2: IntConfig) -> Result<(), Error<E>> {
    let mut r: IoIntCtrl = self.read(Reg::IoIntCtrl).await?;
    r.int1_level = pin1.level;
    r.int1_output = pin1.output;
    r.int1_enable = pin1.enable;
    r.int2_level = pin2.level;
    r.int2_output = pin2.output;
    r.int2_enable = pin2.enable;
    self.write(Reg::IoIntCtrl, r).await
  }

  /// Convenience: set latch and pin configurations in one call.
  pub async fn set_int_config(&mut self, latch: bool, pin1: IntConfig, pin2: IntConfig) -> Result<(), Error<E>> {
    self.set_int_latch(latch).await?;
    self.set_int_pins(pin1, pin2).await
  }

  pub async fn get_int_config(&mut self) -> Result<(IntConfig, IntConfig), Error<E>> {
    let reg: IoIntCtrl = self.read(Reg::IoIntCtrl).await?;
    let p1 = IntConfig { output: reg.int1_output, level: reg.int1_level, enable: reg.int1_enable };
    let p2 = IntConfig { output: reg.int2_output, level: reg.int2_level, enable: reg.int2_enable };
    Ok((p1, p2))
  }

  /// Read `INT_STATUS_INT1` (clear-on-read).
  pub async fn get_int1_status(&mut self) -> Result<IntStatus, Error<E>> {
    self.read(Reg::IntStatusInt1).await
  }

  /// Read `INT_STATUS_INT2` (clear-on-read).
  pub async fn get_int2_status(&mut self) -> Result<IntStatus, Error<E>> {
    self.read(Reg::IntStatusInt2).await
  }

  /// Read `FEATURE_EVENT_EXT`. Only valid when corresponding feature status bits are set.
  pub async fn get_feature_event_ext(&mut self) -> Result<FeatureEventExt, Error<E>> {
    self.read(Reg::FeatureEventExt).await
  }
}

// Hardware-aligned bitfield spanning INT_MAP1..2 (4 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 4)]
pub struct IntMap {
  #[bits(2)]
  pub no_motion: IntPin,
  #[bits(2)]
  pub any_motion: IntPin,
  #[bits(2)]
  pub flat: IntPin,
  #[bits(2)]
  pub orientation: IntPin,
  #[bits(2)]
  pub step_detector: IntPin,
  #[bits(2)]
  pub step_counter: IntPin,
  #[bits(2)]
  pub sig_motion: IntPin,
  #[bits(2)]
  pub tilt: IntPin,
  #[bits(2)]
  pub tap: IntPin,
  #[skip(2)]
  #[bits(2)]
  pub err_status: IntPin,
  #[bits(2)]
  pub temp_data_ready: IntPin,
  #[bits(2)]
  pub gyro_data_ready: IntPin,
  #[bits(2)]
  pub accel_data_ready: IntPin,
  #[bits(2)]
  pub fifo_watermark: IntPin,
  #[bits(2)]
  pub fifo_full: IntPin,
}

// TapEvent collapsed into simple count in FeatureEventExt::Tap
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OrientationPl {
  PortraitUpRight,
  LandscapeLeft,
  PortraitUpDown,
  LandscapeRight,
}

impl From<OrientationPl> for u8 {
  fn from(value: OrientationPl) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for OrientationPl {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x0 => Ok(OrientationPl::PortraitUpRight),
      0x1 => Ok(OrientationPl::LandscapeLeft),
      0x2 => Ok(OrientationPl::PortraitUpDown),
      0x3 => Ok(OrientationPl::LandscapeRight),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Face {
  Up,
  Down,
}

impl From<Face> for u8 {
  fn from(value: Face) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for Face {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x0 => Ok(Face::Up),
      0x1 => Ok(Face::Down),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct FeatureEventExt {
  #[bits(2)]
  pub pl: OrientationPl,
  #[bits(1)]
  pub face: Face,
  #[bits(1)]
  pub s_tap: bool,
  #[bits(1)]
  pub d_tap: bool,
  #[bits(1)]
  pub t_tap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IntConfig {
  pub output: OutputMode,
  pub level: ActiveLevel,
  pub enable: bool,
}

impl Default for IntConfig {
  fn default() -> Self {
    Self { output: OutputMode::OpenDrain, level: ActiveLevel::ActiveHigh, enable: false }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IntPin {
  #[default]
  None = 0x0,
  Int1 = 0x1,
  Int2 = 0x2,
  // Ibi = 0b11,
}

impl From<IntPin> for u8 {
  fn from(value: IntPin) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for IntPin {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0x0 => Ok(IntPin::None),
      0x1 => Ok(IntPin::Int1),
      0x2 => Ok(IntPin::Int2),
      0x3 => Ok(IntPin::None), // treat IBI as None here
      _ => Err(()),
    }
  }
}

/// Represents the possible latching modes for interrupts,
/// mapped to the INT_LATCH (0x54) register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LatchMode {
  NonLatch = 0,
  Latch = 1,
}

/// Configures the pin output mode (push-pull or open-drain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputMode {
  PushPull = 0,
  OpenDrain = 1,
}

impl From<OutputMode> for u8 {
  fn from(value: OutputMode) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for OutputMode {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v & 0x01 {
      0 => Ok(OutputMode::PushPull),
      1 => Ok(OutputMode::OpenDrain),
      _ => Err(()),
    }
  }
}

/// Represents the active level for interrupt pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ActiveLevel {
  ActiveLow = 0,
  ActiveHigh = 1,
}

impl From<ActiveLevel> for u8 {
  fn from(value: ActiveLevel) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for ActiveLevel {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v & 0x01 {
      0 => Ok(ActiveLevel::ActiveLow),
      1 => Ok(ActiveLevel::ActiveHigh),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct IntConfReg {
  #[bits(1)]
  pub latch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct IoIntCtrl {
  // INT1
  #[bits(1)]
  pub int1_level: ActiveLevel,
  #[bits(1)]
  pub int1_output: OutputMode,
  #[bits(1)]
  pub int1_enable: bool,
  #[skip(5)]
  // INT2
  #[bits(1)]
  pub int2_level: ActiveLevel,
  #[bits(1)]
  pub int2_output: OutputMode,
  #[bits(1)]
  pub int2_enable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct IntStatus {
  #[bits(1)]
  pub no_motion: bool,
  #[bits(1)]
  pub any_motion: bool,
  #[bits(1)]
  pub flat: bool,
  #[bits(1)]
  pub orientation: bool,
  #[bits(1)]
  pub step_detector: bool,
  #[bits(1)]
  pub step_counter: bool,
  #[bits(1)]
  pub sig_motion: bool,
  #[bits(1)]
  pub tilt: bool,
  #[bits(1)]
  pub tap: bool,
  #[skip(1)]
  #[bits(1)]
  pub err_status: bool,
  #[bits(1)]
  pub temp_data_ready: bool,
  #[bits(1)]
  pub gyro_data_ready: bool,
  #[bits(1)]
  pub accel_data_ready: bool,
  #[bits(1)]
  pub fifo_watermark: bool,
  #[bits(1)]
  pub fifo_full: bool,
}
