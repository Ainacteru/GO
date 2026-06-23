use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Write TAP configuration (3 words):
  /// - w0: axis/mode/peaks (incl. wait‑for‑timeout)
  /// - w1: peak threshold (10b) + max gesture duration (6b)
  /// - w2: timing nibbles (max_dur_bw_peaks, shock_sett, min_quiet, quiet_after)
  pub async fn set_tap_conf(&mut self, cfg: TapConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::Tap, cfg).await
  }

  /// Read TAP configuration block.
  pub async fn get_tap_conf(&mut self) -> Result<TapConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::Tap).await
  }

  pub async fn enable_tap(&mut self, single: bool, double: bool, triple: bool, cfg: TapConfig) -> Result<(), Error<E>> {
    self.set_tap_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.tap_single = single;
    f.tap_double = double;
    f.tap_triple = triple;
    self.set_enabled_features(f).await
  }
}

/// Tap detector configuration (raw feature fields).
///
/// - axis_sel: accelerometer axis selection for tap detection. Default = 2 (Z).
/// - wait_for_time_out: perform gesture confirmation with the wait time set by
///   `max_gesture_dur`. Default = 1 (enabled).
/// - max_peaks_for_tap: maximum number of zero crossings expected around a tap.
///   Default = 6; range = 0..7.
/// - mode: detection mode (sensitive / normal / robust). Use more robust modes
///   under noisy scenarios to suppress false detection.
/// - tap_peak_threshold: minimum threshold for peak resulting from the tap.
///   Default = 0x2D (≈87.885 mg); LSB ≈ 1.953 mg.
/// - max_gest_duration: maximum duration from first tap within which second/third
///   tap is expected. Default = 16 (≈640 ms); resolution = 40 ms.
/// - max_dur_between_peaks: maximum duration between positive and negative peaks
///   of a tap. Default = 4 (≈20 ms); range up to ≈75 ms; resolution = 5 ms.
/// - tap_shock_settling_dur: maximum duration for which tap impact is observed.
///   Default = 6 (≈30 ms); range up to ≈75 ms; resolution = 5 ms.
/// - min_quite_dur_between_taps: minimum duration between two tap impacts.
///   Default = 8 (≈40 ms); resolution = 5 ms.
/// - quite_time_after_gesture: minimum quiet duration between two gestures.
///   Default = 6 (≈240 ms); resolution = 40 ms.
///
/// Tuning notes:
/// - `peak_thres` scales with accel range and filtering; higher ranges or heavy averaging need higher thresholds.
/// - Multi‑tap recognition needs sufficient quiet and inter‑peak spacing.
/// - `wait_for_timeout=true` improves multi‑tap detection by collecting peaks until the window ends;
///   `false` yields faster single‑tap reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 6)]
pub struct TapConfig {
  /// X/Y/Z selection for evaluation. Default = Z (2).
  #[bits(2)]
  pub axis: TapAxis,
  /// If true, waits the full gesture window before deciding. Default = true.
  pub wait_for_timeout: bool,
  /// 0..7 peaks accepted (typical 1..3 for S/D/T tap). Default = 6.
  #[bits(3)]
  pub max_peaks: u8,
  #[bits(2)]
  pub mode: TapMode,
  #[skip(8)]
  #[bits(10)]
  pub peak_thres: u16, // LSB ≈ 1.953 mg; Default ≈ 0x2D
  #[bits(6)]
  pub max_gest_dur: u8, // LSB = 40 ms; Default = 16 (≈640 ms)
  #[bits(4)]
  pub max_dur_bw_peaks: u8, // LSB = 5 ms; Default = 4 (≈20 ms)
  #[bits(4)]
  pub shock_sett_dur: u8, // LSB = 5 ms; Default = 6 (≈30 ms)
  #[bits(4)]
  pub min_quiet_dur_bw_taps: u8, // LSB = 5 ms; Default = 8 (≈40 ms)
  #[bits(4)]
  pub quiet_time_after_gest: u8, // LSB = 40 ms; Default = 6 (≈240 ms)
}

impl TapConfig {
  pub const fn new(
    axis: TapAxis,
    wait_for_timeout: bool,
    max_peaks: u8,
    mode: TapMode,
    peak_thres: u16,
    max_gest_dur: u8,
    max_dur_bw_peaks: u8,
    shock_sett_dur: u8,
    min_quiet_dur_bw_taps: u8,
    quiet_time_after_gest: u8,
  ) -> Self {
    Self {
      axis,
      wait_for_timeout,
      max_peaks,
      mode,
      peak_thres,
      max_gest_dur,
      max_dur_bw_peaks,
      shock_sett_dur,
      min_quiet_dur_bw_taps,
      quiet_time_after_gest,
    }
  }
}

impl Default for TapConfig {
  /// Official default values from BMI323 tap detector documentation.
  fn default() -> Self {
    Self {
      axis: TapAxis::Z,         // default axis = Z
      wait_for_timeout: true,   // perform gesture confirmation
      max_peaks: 6,             // zero crossings
      mode: TapMode::Mode1,     // maps to "normal" in Bosch docs
      peak_thres: 0x002D,       // ≈87.885 mg
      max_gest_dur: 16,         // 16 × 40 ms ≈ 640 ms
      max_dur_bw_peaks: 4,      // 4 × 5 ms = 20 ms
      shock_sett_dur: 6,        // 6 × 5 ms = 30 ms
      min_quiet_dur_bw_taps: 8, // 8 × 5 ms = 40 ms
      quiet_time_after_gest: 6, // 6 × 40 ms = 240 ms
    }
  }
}

/// Axis selection for tap detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TapAxis {
  X = 0,
  Y = 1,
  Z = 2,
}

impl From<TapAxis> for u8 {
  fn from(value: TapAxis) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for TapAxis {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(TapAxis::X),
      1 => Ok(TapAxis::Y),
      2 => Ok(TapAxis::Z),
      _ => Err(()),
    }
  }
}

/// Tap mode selector (2‑bit variant of the internal classifier).
///
/// Modes trade off robustness vs. responsiveness of the tap recognizer.
/// Official docs describe three conceptual modes (sensitive, normal, robust).
/// This driver exposes four raw codes (0..3) matching hardware; use lower codes
/// for higher sensitivity, higher codes for more robustness to vibration.
///
/// Combine with `max_peaks`, quiet/shock timings, and `wait_for_timeout` to tune
/// for single/double/triple taps and environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TapMode {
  Mode0 = 0,
  Mode1 = 1,
  Mode2 = 2,
  Mode3 = 3,
}

impl From<TapMode> for u8 {
  fn from(value: TapMode) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for TapMode {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(TapMode::Mode0),
      1 => Ok(TapMode::Mode1),
      2 => Ok(TapMode::Mode2),
      3 => Ok(TapMode::Mode3),
      _ => Err(()),
    }
  }
}
