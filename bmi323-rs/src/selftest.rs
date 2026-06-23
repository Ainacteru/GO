use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error, defs::*};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Trigger on-chip self test and read the first result word at `FeatureAddr::StResult`.
  pub async fn run_self_test(&mut self) -> Result<u16, Error<E>> {
    self.write_u16(Reg::Cmd, Command::SelfTestTrigger.into()).await?;
    // Give some time; datasheet recommends a delay. Use a conservative wait.
    self.delay.delay_ms(10).await;
    let mut w = [0u8; 2];
    self.read_feature_bytes(super::FeatureAddr::StResult, &mut w).await?;
    Ok(u16::from_le_bytes(w))
  }

  /// Write raw self-test selection words.
  pub async fn set_self_test_select(&mut self, bytes: &[u8]) -> Result<(), Error<E>> {
    self.write_feature_bytes(super::FeatureAddr::StSelect, bytes).await
  }

  /// Read raw self-test selection words.
  pub async fn get_self_test_select(&mut self, out: &mut [u8]) -> Result<(), Error<E>> {
    self.read_feature_bytes(super::FeatureAddr::StSelect, out).await
  }
}
