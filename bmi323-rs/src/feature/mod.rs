//! Feature Engine helpers and per‑feature APIs.
//!
//! The BMI323 includes a small on‑chip "feature engine" that implements
//! wake‑up/orientation/flat/tap/step and related algorithms. This module
//! provides:
//! - One‑liners to enable/disable the engine.
//! - Read/write accessors for the feature configuration memory.
//! - Strongly‑typed config structs for individual features (see submodules).
//!
//! Quick start:
//! 1) Enable the engine: `bmi.enable_feature_engine().await?`.
//! 2) Choose features and map interrupts (see `interrupt` module).
//! 3) Configure each feature via its submodule (e.g., `tilt`, `orientation`).
//!
//! Tip: Many configs implement `Default` with sensible values from the
//! official reference. Start there, then tweak thresholds for your product.

use embedded_hal_async::{delay::DelayNs, i2c::*};

use super::{Bmi323, Error, defs::*};

// Submodules with per-feature configuration APIs
pub mod any_no_motion;
pub mod axis_remap;
pub mod flat;
pub mod orientation;
pub mod sig_motion;
pub mod step;
pub mod tap;
pub mod tilt;

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Enable the BMI323 feature engine.
  ///
  /// Sequence (per datasheet):
  /// - Program `FEATURE_IO2` with the init value.
  /// - Apply via `FEATURE_IO_STATUS`.
  /// - Set `FEATURE_CTRL.engine_en`.
  /// - Poll until `FEATURE_IO1.error_status == Activated`.
  ///
  /// After activation, feature configuration and interrupt routing can be
  /// programmed safely. Call this once after reset or power‑up.
  pub async fn enable_feature_engine(&mut self) -> Result<(), Error<E>> {
    self.write_bytes(Reg::FeatureIo2, &[0x2c, 0x01]).await?;
    self.write_bytes(Reg::FeatureIoStatus, &[1, 0]).await?;
    self.write_bytes(Reg::FeatureCtrl, &[1, 0]).await?;

    let mut tries = 0;
    loop {
      self.delay.delay_us(100_000).await;

      let v: FeatureIo1 = self.read(Reg::FeatureIo1).await?;
      if v.error_status == FeatureIoError::Activated {
        break;
      }

      tries += 1;
      if tries > 10 {
        return Err(Error::Init);
      }
    }

    Ok(())
  }

  /// Disable the feature engine. A soft reset or power‑cycle is required before enabling again.
  pub async fn disable_feature_engine(&mut self) -> Result<(), Error<E>> {
    self.write_u16(Reg::FeatureCtrl, 0).await
  }

  pub async fn is_feature_engine_enabled(&mut self) -> Result<bool, Error<E>> {
    Ok(self.read_u16(Reg::FeatureCtrl).await? == 1)
  }

  pub(crate) async fn write_feature<const N: usize, T>(&mut self, addr: FeatureAddr, v: T) -> Result<(), Error<E>>
  where
    T: TryInto<[u8; N]>,
  {
    let bytes = v.try_into().map_err(|_| Error::Data)?;
    self.write_feature_bytes(addr, &bytes).await
  }

  pub(crate) async fn write_feature_bytes(&mut self, addr: FeatureAddr, v: &[u8]) -> Result<(), Error<E>> {
    self.wait_feature_data_ready().await?;
    self.write_bytes(Reg::FeatureDataAddr, &[addr as u8, 0]).await?;
    self.write_bytes(Reg::FeatureDataTx, v).await?;
    Ok(())
  }

  /// Read and unpack a feature configuration block into a typed value.
  pub(crate) async fn read_feature<const N: usize, T>(&mut self, addr: FeatureAddr) -> Result<T, Error<E>>
  where
    T: TryFrom<[u8; N]>,
  {
    let mut bytes = [0u8; N];
    self.read_feature_bytes(addr, &mut bytes).await?;
    bytes.try_into().map_err(|_| Error::Data)
  }

  /// Read a contiguous block of feature words into `out`.
  pub(crate) async fn read_feature_bytes(&mut self, addr: FeatureAddr, out: &mut [u8]) -> Result<(), Error<E>> {
    // Per datasheet §6.2 (Extended Register Map): see comment in write_feature_bytes.
    if out.len() % 2 != 0 {
      return Err(Error::Data);
    }

    self.wait_feature_data_ready().await?;
    self.write_bytes(Reg::FeatureDataAddr, &[addr as u8, 0]).await?;
    self.read_bytes(Reg::FeatureDataTx, out).await
  }

  /// Read the current feature enable bitmask (`FEATURE_IO0`).
  pub(crate) async fn get_enabled_features(&mut self) -> Result<Features, Error<E>> {
    self.read(Reg::FeatureIo0).await
  }

  /// Write the feature enable bitmask (`FEATURE_IO0`).
  ///
  /// Example: enable Any‑motion on all axes and Orientation.
  ///
  /// ```no_run
  /// # async fn demo<E>(bmi: &mut bmi323::Bmi323<impl embedded_hal_async::i2c::I2c<embedded_hal_async::i2c::SevenBitAddress, Error=E>, impl embedded_hal_async::delay::DelayNs>) -> Result<(), bmi323::Error<E>> {
  /// let mut feats = bmi323::feature::Features::none();
  /// feats.any_motion_x = true;
  /// feats.any_motion_y = true;
  /// feats.any_motion_z = true;
  /// feats.orientation = true;
  /// bmi.set_enabled_features(feats).await?;
  /// # Ok(()) }
  /// ```
  pub(crate) async fn set_enabled_features(&mut self, v: Features) -> Result<(), Error<E>> {
    self.write(Reg::FeatureIo0, v).await?;
    self.write_u16(Reg::FeatureIoStatus, 0x1).await
  }

  pub async fn get_feature_data_status(&mut self) -> Result<FeatureDataStatus, Error<E>> {
    self.read(Reg::FeatureDataStatus).await
  }

  pub async fn get_feature_engine_status(&mut self) -> Result<FeatureEngineStatus, Error<E>> {
    self.read(Reg::FeatureEngineStatus).await
  }

  /// Internal: wait until the feature data interface is ready for I/O.
  ///
  /// Datasheet §6.2 requires `FEATURE_DATA_STATUS.data_tx_ready == 1` before
  /// setting the extended address and performing a feature read/write.
  async fn wait_feature_data_ready(&mut self) -> Result<(), Error<E>> {
    let mut tries = 0;
    loop {
      let status = self.get_feature_data_status().await?;
      if status.out_of_bound_err {
        // Previous operation signaled an out‑of‑bound access; propagate as data error.
        return Err(Error::Data);
      }
      if status.data_tx_ready {
        return Ok(());
      }
      if tries > 100 {
        return Err(Error::Data);
      }
      self.delay.delay_ms(2).await;
      tries += 1;
    }
  }
}

/// Feature memory base addresses (used with FEATURE_DATA_ADDR/TX).
#[repr(u8)]
pub enum FeatureAddr {
  AxisRemap = 0x03,
  AnyMotion = 0x05,
  NoMotion = 0x08,
  Flat = 0x0B,
  SigMotion = 0x0D,
  StepCnt = 0x10,
  Orient = 0x1C,
  Tap = 0x1E,
  Tilt = 0x21,
  AltAutoConfig = 0x23,
  StResult = 0x24,
  StSelect = 0x25,
  GyroScSelect = 0x26,
  GyroScStConf = 0x27,
  GyroScStCoefficients = 0x28,
}

impl From<FeatureAddr> for u8 {
  #[inline]
  fn from(a: FeatureAddr) -> Self {
    a as u8
  }
}

/// Suppress state changes when the device is in strong motion.
///
/// The feature engine can optionally block state transitions if the device is
/// moving too much. This reduces flicker and false positives on vibrating rigs.
/// - `Disabled`: never block.
/// - `AccelOver1p5g`: block if |a| on any axis > 1.5 g.
/// - `AccelOver1p5gOrHalfSlope`: block if |a| > 1.5 g OR the internal slope
///   estimate exceeds 0.5× the configured slope threshold.
/// - `AccelOver1p5gOrSlope`: like above, but uses the full slope threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BlockingRule {
  Disabled = 0,
  AccelOver1p5g = 1,
  AccelOver1p5gOrHalfSlope = 2,
  AccelOver1p5gOrSlope = 3,
}

impl From<BlockingRule> for u8 {
  fn from(value: BlockingRule) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for BlockingRule {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(BlockingRule::Disabled),
      1 => Ok(BlockingRule::AccelOver1p5g),
      2 => Ok(BlockingRule::AccelOver1p5gOrHalfSlope),
      3 => Ok(BlockingRule::AccelOver1p5gOrSlope),
      _ => Err(()),
    }
  }
}

/// Helper for human‑readable timing used by Any‑/No‑motion.
///
/// - `duration_s`: time the condition must hold before asserting.
/// - `wait_time_s`: extra delay before clearing (debounce).
///
/// Both map to 20 ms ticks in hardware; values are clamped to the field width
/// of the underlying feature.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MotionTiming {
  pub duration_s: f32,
  pub wait_time_s: f32,
}

impl MotionTiming {
  pub const fn new(duration_s: f32, wait_time_s: f32) -> Self {
    Self { duration_s, wait_time_s }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[packbits::pack(bytes = 2)]
struct FeatureIo1 {
  #[bits(4)]
  pub error_status: FeatureIoError,
  pub sc_st_complete: bool,
  pub gyro_sc_result: bool,
  pub st_result: bool,
  pub sample_rate_err: bool,
  #[skip(2)]
  pub axis_map_complete: bool,
  #[bits(2)]
  pub engine_state: FeatureEngineState,
}

/// Decoded values for `FEATURE_IO1.error_status` (bits[3:0]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FeatureIoError {
  Inactive = 0x0,                 // Feature engine still inactive
  Activated = 0x1,                // Feature engine activated
  ConfigStringWrong = 0x3,        // Configuration string download wrong
  NoError = 0x5,                  // No error
  AxisMapCmdNotProcessed = 0x6,   // Axis map command not processed (sensor active or SC/ST ongoing)
  I3cTcSyncError = 0x8,           // I3C TC-sync error (active/ongoing conflicts)
  OngoingScOrStAborted = 0x9,     // Ongoing SC/ST aborted or due to TC-sync request
  ScCmdIgnored = 0xA,             // SC command ignored (SC/ST or TC-sync ongoing)
  StCmdIgnored = 0xB,             // ST command ignored (SC/ST or TC-sync ongoing)
  ScCmdNotProcessed = 0xC,        // SC not processed (pre-conditions not met)
  AutoModeIllegalConfig = 0xD,    // Auto-mode/illegal sensor config change during SC/ST
  TcSyncEnableWhileSt = 0xE,      // TC-sync enable while self-test ongoing
  IllegalConfigWhileTcSync = 0xF, // Illegal sensor config while TC-sync active
}

impl From<FeatureIoError> for u8 {
  fn from(value: FeatureIoError) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for FeatureIoError {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    use FeatureIoError::*;
    Ok(match v {
      0x0 => Inactive,
      0x1 => Activated,
      0x3 => ConfigStringWrong,
      0x5 => NoError,
      0x6 => AxisMapCmdNotProcessed,
      0x8 => I3cTcSyncError,
      0x9 => OngoingScOrStAborted,
      0xA => ScCmdIgnored,
      0xB => StCmdIgnored,
      0xC => ScCmdNotProcessed,
      0xD => AutoModeIllegalConfig,
      0xE => TcSyncEnableWhileSt,
      0xF => IllegalConfigWhileTcSync,
      _ => return Err(()),
    })
  }
}

/// Values for `FEATURE_IO1.state` (bits[12:11]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FeatureEngineState {
  FeatureMode = 0x0,
  GyroScRunning = 0x1,
  SelfTestMode = 0x2,
  ErrorMode = 0x3,
}

impl From<FeatureEngineState> for u8 {
  fn from(value: FeatureEngineState) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for FeatureEngineState {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    use FeatureEngineState::*;
    Ok(match v & 0x03 {
      0x0 => FeatureMode,
      0x1 => GyroScRunning,
      0x2 => SelfTestMode,
      0x3 => ErrorMode,
      _ => unreachable!(),
    })
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[packbits::pack(bytes = 2)]
pub struct Features {
  /// Assert No‑motion when the X‑axis slope stays below the programmed threshold.
  pub no_motion_x: bool,
  /// Assert No‑motion when the Y‑axis slope stays below the programmed threshold.
  pub no_motion_y: bool,
  /// Assert No‑motion when the Z‑axis slope stays below the programmed threshold.
  pub no_motion_z: bool,
  /// Assert Any‑motion when the X‑axis slope exceeds the programmed threshold.
  pub any_motion_x: bool,
  /// Assert Any‑motion when the Y‑axis slope exceeds the programmed threshold.
  pub any_motion_y: bool,
  /// Assert Any‑motion when the Z‑axis slope exceeds the programmed threshold.
  pub any_motion_z: bool,
  /// Face‑up/face‑down detection.
  pub flat: bool,
  /// Portrait/Landscape and face state detection.
  pub orientation: bool,
  /// Step detector (single‑step events).
  pub step_detector: bool,
  /// Step counter (accumulated steps).
  pub step_counter: bool,
  /// Significant Motion (activity over a window).
  pub sig_motion: bool,
  /// Tilt detector.
  pub tilt: bool,
  /// Single tap event.
  pub tap_single: bool,
  /// Double tap event.
  pub tap_double: bool,
  /// Triple tap event.
  pub tap_triple: bool,
}

impl Features {
  /// All features disabled (convenient starting point).
  pub const fn none() -> Self {
    Self {
      no_motion_x: false,
      no_motion_y: false,
      no_motion_z: false,
      any_motion_x: false,
      any_motion_y: false,
      any_motion_z: false,
      flat: false,
      orientation: false,
      step_detector: false,
      step_counter: false,
      sig_motion: false,
      tilt: false,
      tap_single: false,
      tap_double: false,
      tap_triple: false,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[packbits::pack(bytes = 2)]
pub struct FeatureDataStatus {
  pub out_of_bound_err: bool,
  pub data_tx_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[packbits::pack(bytes = 2)]
pub struct FeatureEngineStatus {
  /// Feature engine is in halt or sleep state.
  pub halt_or_sleep: bool,
  /// Ongoing transfer of data to/from the feature engine.
  pub overload: bool,
  #[skip(1)]
  /// DMA controller has started DMA and DMA is in progress.
  pub data_tx_active: bool,
  /// Feature engine was disabled by host. Perform a soft reset to re-enable feature engine.
  pub disabled_by_host: bool,
  /// Feature engine watchdog timer has expired. Perform a soft reset to re-enable feature engine.
  pub watchdog_not_ack: bool,
}
