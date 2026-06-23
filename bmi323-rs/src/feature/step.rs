use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Read 32‑bit step count from feature output words at `FeatureAddr::StepCnt`.
  ///
  /// Context:
  /// - Pair with an interrupt map on Step Counter (watermark) or Step Detector
  ///   for timely updates without polling.
  pub async fn get_step_count(&mut self) -> Result<u32, Error<E>> {
    let mut bytes = [0u8; 4];
    self.read_feature_bytes(super::FeatureAddr::StepCnt, &mut bytes).await?;
    Ok(u32::from_le_bytes(bytes))
  }

  /// Set step counter watermark (lower 10 bits of word 0).
  ///
  /// Context:
  /// - When the count reaches the configured watermark, an interrupt can be
  ///   generated (if mapped) to service reads in batches.
  /// - Watermark holds an implicit 20× factor (resolution = 20 steps). Range
  ///   0..20460 steps (10‑bit raw field × 20). A raw value of 0 disables output.
  pub async fn set_step_watermark(&mut self, wm: u16) -> Result<(), Error<E>> {
    let mut w0: StepWord0 = self.read_feature(super::FeatureAddr::StepCnt).await?;
    w0.watermark = core::cmp::min(wm, (1u16 << 10) - 1);
    self.write_feature(super::FeatureAddr::StepCnt, w0).await
  }

  /// Reset the step counter by setting the reset bit in word 0.
  ///
  /// Context:
  /// - Use when starting a session or after persisting the current count.
  pub async fn reset_step_counter(&mut self) -> Result<(), Error<E>> {
    let mut w0: StepWord0 = self.read_feature(super::FeatureAddr::StepCnt).await?;
    w0.reset = true;
    self.write_feature(super::FeatureAddr::StepCnt, w0).await
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct StepWord0 {
  #[bits(10)]
  pub watermark: u16,
  pub reset: bool,
}
