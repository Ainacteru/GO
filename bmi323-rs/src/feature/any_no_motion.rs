use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_any_motion_conf(&mut self, cfg: AnyNoMotionConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::AnyMotion, cfg).await
  }

  pub async fn get_any_motion_conf(&mut self) -> Result<AnyNoMotionConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::AnyMotion).await
  }

  pub async fn enable_any_motion(&mut self, x: bool, y: bool, z: bool, cfg: AnyNoMotionConfig) -> Result<(), Error<E>> {
    self.set_any_motion_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.any_motion_x = x;
    f.any_motion_y = y;
    f.any_motion_z = z;
    self.set_enabled_features(f).await
  }

  pub async fn set_no_motion_conf(&mut self, cfg: AnyNoMotionConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::NoMotion, cfg).await
  }

  pub async fn get_no_motion_conf(&mut self) -> Result<AnyNoMotionConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::NoMotion).await
  }

  pub async fn enable_no_motion(&mut self, x: bool, y: bool, z: bool, cfg: AnyNoMotionConfig) -> Result<(), Error<E>> {
    self.set_no_motion_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.no_motion_x = x;
    f.no_motion_y = y;
    f.no_motion_z = z;
    self.set_enabled_features(f).await
  }
}

/// Any-/No‑motion configuration (raw feature fields; no unit conversion).
///
/// - slope_thres: minimum slope of acceleration signal for motion detection.
///   Default = 10; range = 0..4095; unit = 1.953 mg (LSB = 1/512 g).
/// - acc_ref_update: mode of the acceleration reference update (0: fixed, 1: update).
/// - hysteresis: hysteresis for the slope of the acceleration signal.
///   Default = 2; range = 0..1023; unit = 1.953 mg (LSB = 1/512 g).
/// - duration: number of consecutive 20 ms samples for which the threshold
///   condition must hold for interrupt assertion. Default = 10; unit = 20 ms.
/// - wait_time: wait time for clearing the event after slope is below threshold.
///   Default = 3; range = 0..7; unit = 20 ms.
///
/// Behavior:
/// - Any‑motion asserts when the internal slope estimate exceeds `slope_threshold`
///   for at least `duration × 20 ms`.
/// - No‑motion asserts when the slope stays below `slope_threshold` for at least
///   `duration × 20 ms`.
/// - `wait_time` adds a post‑condition delay before an event clears to reduce chatter.
/// - Results depend on accel ODR/bandwidth/averaging; stronger filtering reduces noise
///   but can delay detection or hide short bursts.
///
/// Raw field widths (clamped by driver):
/// - `slope_threshold`: 12 bits (0..4095) → g ≈ raw/512
/// - `hysteresis`: 10 bits (0..1023) → g ≈ raw/512
/// - `duration`: 13 bits, LSB = 20 ms
/// - `wait_time`: 3 bits, LSB = 20 ms
///
/// Tips:
/// - Use a small but non‑zero `hysteresis` to avoid re‑triggering on noise.
/// - For “no‑motion”, consider longer `duration` to avoid chattering when near threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 6)]
pub struct AnyNoMotionConfig {
  /// Slope threshold (12 bits, LSB = 1/512 g). g ≈ raw/512. Default = 10.
  /// Examples: 256 → 0.50 g, 512 → 1.00 g, 1024 → 2.00 g, 2048 → 4.00 g, 4095 → ≈7.998 g.
  #[bits(12)]
  pub slope_threshold: u16,
  /// Allow hardware to update its internal acceleration reference (baseline).
  #[bits(1)]
  pub acc_ref_update: bool,
  /// Hysteresis (10 bits, LSB = 1/512 g). g ≈ raw/512. Default = 2.
  /// Examples: 256 → 0.50 g, 512 → 1.00 g, 768 → 1.50 g, 1023 → ≈1.998 g.
  #[skip(3)]
  #[bits(10)]
  pub hysteresis: u16,
  /// Condition duration (13 bits, LSB = 20 ms). seconds ≈ raw/50. Default ≈ 10.
  /// Examples: 50 → 1.0 s, 25 → 0.5 s, 10 → 0.2 s.
  #[skip(6)]
  #[bits(13)]
  pub duration: u16,
  /// Post‑condition wait (3 bits, LSB = 20 ms). seconds ≈ raw/50. Default = 3.
  /// Examples: 1 → 20 ms, 3 → 60 ms, 7 → 140 ms (max).
  #[bits(3)]
  pub wait_time: u8,
}

impl AnyNoMotionConfig {
  pub const fn new(slope_threshold: u16, hysteresis: u16, duration: u16, wait_time: u8) -> Self {
    Self { slope_threshold, acc_ref_update: false, hysteresis, duration, wait_time }
  }
}

impl Default for AnyNoMotionConfig {
  /// Official default values where specified by Bosch; unspecified fields use sensible defaults.
  fn default() -> Self {
    Self {
      slope_threshold: 10,   // 10 LSB ≈ 19.53 mg
      acc_ref_update: false, // not specified in docs; default to fixed reference
      hysteresis: 2,         // 2 LSB ≈ 3.906 mg
      duration: 10,          // 10 × 20 ms = 200 ms
      wait_time: 3,          // 3 × 20 ms = 60 ms
    }
  }
}
