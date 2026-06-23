use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_orientation_conf(&mut self, cfg: OrientationConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::Orient, cfg).await
  }

  pub async fn get_orientation_conf(&mut self) -> Result<OrientationConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::Orient).await
  }

  pub async fn enable_orientation(&mut self, cfg: OrientationConfig) -> Result<(), Error<E>> {
    self.set_orientation_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.orientation = true;
    self.set_enabled_features(f).await
  }
}

/// Orientation (portrait/landscape, face‑up/down) configuration.
///
/// - upside/down detection (`ud_en`): enables upside/down detection if set to 1.
/// - mode: symmetry/asymmetry of the classifier thresholds.
///   Values 0 or 3 → symmetrical; 1 → high asymmetrical; 2 → low asymmetrical.
/// - blocking: enable to suppress orientation interrupts under dynamics.
///   Default = 3 (most restrictive blocking).
/// - theta: threshold angle used in blocking mode.
///   Relation: `theta = 64 * (tan(angle)^2)`; Default = 39; range = 0..63.
/// - hysteresis: hysteresis of acceleration in orientation change detection.
///   Default = 32; range = 0..255; unit ≈ 1.953 mg (LSB = 1/512 g).
/// - hold_time: minimum duration the device shall be in new orientation for change detection.
///   Default = 5; unit = 20 ms.
/// - slope_thres: minimum slope between consecutive acceleration samples to prevent
///   change of orientation during large movement. Default = 205; range = 0..255;
///   unit ≈ 1.953 mg (LSB = 1/512 g).
///
/// Notes:
/// - Angle uses the same non‑linear mapping as `flat` (degrees ≈ atan(sqrt(raw/64)) × 180/π).
/// - Hold time is quantized in 20 ms steps. Use small but non‑zero values to avoid boundary flicker.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 4)]
pub struct OrientationConfig {
  /// Enable face‑up/face‑down classification (false: ignore face state).
  pub ud_en: bool,
  /// Symmetry/asymmetry mode of the orientation classifier. Higher modes make
  /// the classifier “stickier” by using different enter/exit thresholds.
  #[bits(2)]
  pub mode: OrientationMode,
  /// Debounce level at the decision boundary.
  #[bits(2)]
  pub blocking: super::BlockingRule,
  /// Theta (0..=63). Degrees ≈ atan(sqrt(raw/64)) × 180/π. Default = 39.
  /// Reference:
  /// - raw 0  → 0.0°
  /// - raw 16 → ≈17.7°
  /// - raw 32 → ≈30.0°
  /// - raw 48 → ≈39.1°
  /// - raw 63 → ≈44.8°
  #[bits(6)]
  pub theta: u8,
  /// Hold time (ticks). LSB = 20 ms → seconds ≈ raw/50. Default = 5. (5 bits here)
  /// Handy reference points:
  /// - raw 1  → 20 ms
  /// - raw 5  → 100 ms
  /// - raw 10 → 200 ms
  /// - raw 25 → 500 ms (near the upper end for this 5‑bit field)
  #[bits(5)]
  pub hold_time: u8,
  /// Slope threshold (LSB = 1/512 g). g ≈ raw/512. Default = 205 (≈400.365 mg).
  /// Examples: 1 → ≈0.002 g, 10 → ≈0.020 g, 50 → ≈0.098 g, 255 → ≈0.498 g.
  pub slope_threshold: u8,
  /// Slope hysteresis (LSB = 1/512 g). g ≈ raw/512.
  /// Examples: 1 → ≈0.002 g, 10 → ≈0.020 g, 50 → ≈0.098 g, 255 → ≈0.498 g.
  pub hysterisis: u8,
}

impl OrientationConfig {
  /// Construct using degrees/seconds and g‑units.
  pub const fn new(
    ud_en: bool,
    mode: OrientationMode,
    blocking: super::BlockingRule,
    theta: u8,
    hold_time: u8,
    slope_threshold: u8,
    hysterisis: u8,
  ) -> Self {
    Self { ud_en, mode, blocking, theta, hold_time, slope_threshold, hysterisis }
  }
}

impl Default for OrientationConfig {
  /// Official defaults where specified; unspecified fields use conservative values.
  fn default() -> Self {
    Self {
      ud_en: true,                                         // not specified; enable face classification by default
      mode: OrientationMode::Symmetric,                    // symmetrical thresholds
      blocking: super::BlockingRule::AccelOver1p5gOrSlope, // most restrictive (3) per docs
      theta: 39,                                           // per docs mapping
      hold_time: 5,                                        // 5 × 20 ms = 100 ms
      slope_threshold: 205,                                // ≈400.365 mg
      hysterisis: 32,                                      // ≈62.5 mg (LSB=1.953 mg)
    }
  }
}

/// Orientation mode: symmetric vs. asymmetric enter/exit thresholds.
///
/// Higher asymmetry makes the classifier “stickier” (different angles to enter
/// and exit), which helps reduce chatter near the decision boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OrientationMode {
  Symmetric = 0,
  MildAsymmetric = 1,
  StrongAsymmetric = 2,
}

impl From<OrientationMode> for u8 {
  fn from(value: OrientationMode) -> Self {
    value as u8
  }
}

impl TryFrom<u8> for OrientationMode {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(OrientationMode::Symmetric),
      1 => Ok(OrientationMode::MildAsymmetric),
      2 => Ok(OrientationMode::StrongAsymmetric),
      _ => Err(()),
    }
  }
}
