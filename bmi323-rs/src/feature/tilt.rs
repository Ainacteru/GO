use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  pub async fn set_tilt_conf(&mut self, cfg: TiltConfig) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::Tilt, cfg).await
  }

  pub async fn get_tilt_conf(&mut self) -> Result<TiltConfig, Error<E>> {
    self.read_feature(super::FeatureAddr::Tilt).await
  }

  pub async fn enable_tilt(&mut self, cfg: TiltConfig) -> Result<(), Error<E>> {
    self.set_tilt_conf(cfg).await?;
    let mut f = self.get_enabled_features().await?;
    f.tilt = true;
    self.set_enabled_features(f).await
  }
}

/// Tilt detector configuration (raw feature fields).
///
/// - segment_size: duration for which the acceleration vector is averaged to be
///   the reference vector. Default = 100; range = 0..255; unit = 20 ms.
/// - minimum tilt angle: minimum angle by which the device shall be tilted for event
///   detection. Angle is computed as `256 * cos(angle)`; Default = 210; range = 0..255.
/// - beta accel mean: exponential smoothing coefficient for computing low‑pass mean of
///   the acceleration vector. Coefficient is computed as `beta * 65536`.
///   Default = 61545; range = 0..65535.
///
/// Notes:
/// - Encodings are non‑linear. Configure against target motions and ODR/bandwidth.
/// - Larger `beta_acc_mean` reduces chatter on vibration at the cost of latency.
/// - Excessive averaging or very low ODR can hide short tilts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 4)]
pub struct TiltConfig {
  /// Segment size (encoded, 8‑bit). LSB = 20 ms. Default = 100.
  pub segment_size: u8,
  /// Minimum tilt angle (encoded). `raw = 256 * cos(angle)`; Default = 210.
  pub min_tilt_angle: u8,
  /// Beta for the accelerometer mean low‑pass (encoded). `raw = beta * 65536`.
  /// Default = 61545.
  pub beta_acc_mean: u16,
}

impl TiltConfig {
  pub const fn new(segment_size: u8, min_tilt_angle: u8, beta_acc_mean: u16) -> Self {
    Self { segment_size, min_tilt_angle, beta_acc_mean }
  }
}

impl Default for TiltConfig {
  fn default() -> Self {
    Self { segment_size: 100, min_tilt_angle: 210, beta_acc_mean: 61545 }
  }
}
