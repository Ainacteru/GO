use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_sign_motion_conf(&mut self, cfg: SigMotionConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::SigMotion, cfg).await
  }

  pub async fn get_sign_motion_conf(&mut self) -> Result<SigMotionConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::SigMotion).await
  }

  pub async fn enable_sig_motion(&mut self, cfg: SigMotionConfig) -> Result<(), Error<E>> {
    self.set_sign_motion_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.sig_motion = true;
    self.set_enabled_features(f).await
  }
}

/// Significant Motion configuration (raw feature fields).
///
/// Detects sustained motion using amplitude (P2P) and activity (MCR):
/// - block_size: duration after which significant motion interrupt is triggered.
///   Expressed in 50 Hz samples (20 ms). Default = 0xFA (250) → 5 s.
/// - peak_2_peak_min (p2p_min): minimum peak‑to‑peak acceleration magnitude.
///   Default = 0x26 (≈74.214 mg); range = 0..1023; unit ≈ 1.953 mg (LSB = 1/512 g).
/// - mcr_min: minimum number of mean crossings per second in accel magnitude.
///   Default = 17; range = 0..63.
/// - peak_2_peak_max (p2p_max): maximum peak‑to‑peak acceleration magnitude.
///   Default = 0x253 (≈1162.035 mg); range = 0..1023; unit ≈ 1.953 mg.
/// - mcr_max: maximum number of mean crossings per second in accel magnitude.
///   Default = 17; range = 0..63.
///
/// Behavior:
/// - Asserts when both P2P and MCR metrics remain within the configured ranges.
/// - Larger `block_size` smooths sporadic motion but increases detection latency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 6)]
pub struct SigMotionConfig {
  /// Block size (encoded) used by the internal estimator. LSB = 20 ms. Default = 0xFA (≈5 s).
  pub block_size: u16,
  /// Minimum peak‑to‑peak amplitude (lower 10 bits, encoded). LSB ≈ 1.953 mg. Default = 0x26.
  #[bits(10)]
  pub p2p_min: u16,
  /// Minimum mean‑crossing rate (upper 6 bits, 0..63). Default = 17.
  #[bits(6)]
  pub mcr_min: u8,
  /// Maximum peak‑to‑peak amplitude (lower 10 bits, encoded). LSB ≈ 1.953 mg. Default = 0x253.
  #[bits(10)]
  pub p2p_max: u16,
  /// Maximum mean‑crossing rate (upper 6 bits, 0..63). Default = 17.
  #[bits(6)]
  pub mcr_max: u8,
}

impl SigMotionConfig {
  pub const fn new(block_size: u16, p2p_min: u16, mcr_min: u8, p2p_max: u16, mcr_max: u8) -> Self {
    Self { block_size, p2p_min, mcr_min, p2p_max, mcr_max }
  }
}

impl Default for SigMotionConfig {
  /// Official default values from BMI323 reference implementation.
  fn default() -> Self {
    Self { block_size: 0x00FA, p2p_min: 0x0026, mcr_min: 17, p2p_max: 0x0253, mcr_max: 17 }
  }
}
