use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_flat_conf(&mut self, cfg: FlatConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::Flat, cfg).await
  }

  pub async fn get_flat_conf(&mut self) -> Result<FlatConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::Flat).await
  }

  pub async fn enable_flat(&mut self, cfg: FlatConfig) -> Result<(), Error<E>> {
    self.set_flat_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.flat = true;
    self.set_enabled_features(f).await
  }
}

/// Flat (device face‑up/face‑down) configuration.
///
/// - theta: tilt angle used for detecting flat position.
///   Relation: `theta = 64 * (tan(angle)^2)`; Default = 8 (~20°).
/// - hysteresis (theta): hysteresis for theta flat detection.
///   Default = 9 (~2.5° w.r.t. default theta).
/// - blocking: sets blocking mode. If blocking is set, no Flat interrupt will be
///   triggered under strong dynamics. Default = 2 (most restrictive).
/// - hold_time: duration in 50 Hz samples for which the condition has to be
///   respected. Default = 32 (640 ms). Range nominally up to ~5.1 s. LSB = 20 ms.
/// - slope_thres: minimum slope between consecutive acceleration samples to prevent
///   flat state changes during large movement. Default = 0xCD (≈400.365 mg).
///
/// Notes and tuning:
/// - `theta` trades sensitivity vs stability: smaller values declare flat earlier; larger values allow more tilt.
/// - `hold_time` debounces changes (e.g., on a desk). LSB = 20 ms.
/// - Use `blocking` and slope threshold/hysteresis when the device vibrates to reduce chattering.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 4)]
pub struct FlatConfig {
  /// Theta (0..=63). Degrees ≈ atan(sqrt(raw/64)) × 180/π. Default = 8 (~20°).
  /// Larger values allow more tilt; >63 saturates. Handy reference points:
  /// - raw 0  → 0.0°
  /// - raw 16 → ≈17.7°
  /// - raw 32 → ≈30.0°
  /// - raw 48 → ≈39.1°
  /// - raw 63 → ≈44.8°
  #[bits(6)]
  pub theta: u8,
  /// Blocking rule under dynamics.
  #[bits(2)]
  pub blocking: super::BlockingRule,
  /// Hold time (ticks). LSB = 20 ms → seconds ≈ raw/50. Default = 32 (640 ms).
  /// Handy reference points:
  /// - raw 1  → 20 ms
  /// - raw 5  → 100 ms
  /// - raw 10 → 200 ms
  /// - raw 25 → 500 ms
  /// - raw 50 → 1.00 s
  pub hold_time: u8,
  /// Slope threshold (LSB = 1/512 g). g ≈ raw/512. Default = 0xCD (≈400.365 mg).
  /// Examples: 1 → ≈0.002 g, 10 → ≈0.020 g, 50 → ≈0.098 g, 255 → ≈0.498 g.
  pub slope_threshold: u8,
  /// Hysteresis (LSB = 1/512 g). g ≈ raw/512.
  /// Examples: 1 → ≈0.002 g, 10 → ≈0.020 g, 50 → ≈0.098 g, 255 → ≈0.498 g.
  pub hysterisis: u8,
}

impl FlatConfig {
  /// Construct with a raw hold time value (LSB = 20 ms). For reference:
  /// - `raw = 1`  → 20 ms
  /// - `raw = 5`  → 100 ms
  /// - `raw = 10` → 200 ms
  /// - `raw = 25` → 500 ms
  /// - `raw = 50` → 1.00 s
  pub const fn new(
    theta: u8,
    blocking: super::BlockingRule,
    hold_time: u8,
    slope_threshold: u8,
    hysterisis: u8,
  ) -> Self {
    Self { theta, blocking, hold_time, slope_threshold, hysterisis }
  }
}

impl Default for FlatConfig {
  /// Official defaults where specified; unspecified fields use conservative values.
  fn default() -> Self {
    Self {
      theta: 8,                                                // ≈20°
      blocking: super::BlockingRule::AccelOver1p5gOrHalfSlope, // default 2
      hold_time: 32,                                           // 32 × 20 ms = 640 ms
      slope_threshold: 0xCD,                                   // ≈400.365 mg
      hysterisis: 0,                                           // not specified in docs
    }
  }
}
