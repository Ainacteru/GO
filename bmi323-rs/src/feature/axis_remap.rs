use embedded_hal_async::{delay::DelayNs, i2c::*};

use crate::{Bmi323, Error, defs::*};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Program axis remap (order and sign) via feature block, then apply.
  pub async fn set_axis_remap(&mut self, map: AxisRemap) -> Result<(), Error<E>> {
    self.write_feature(super::FeatureAddr::AxisRemap, map).await?;
    self.write_u16(Reg::Cmd, Command::AxisMapUpdate.into()).await?;
    self.wait_axis_map_complete().await?;
    Ok(())
  }

  /// Read current axis remap configuration from the feature block.
  pub async fn get_axis_remap(&mut self) -> Result<AxisRemap, Error<E>> {
    self.read_feature(super::FeatureAddr::AxisRemap).await
  }

  /// Poll `FEATURE_ENGINE_STATUS` for axis-map complete bit.
  async fn wait_axis_map_complete(&mut self) -> Result<(), Error<E>> {
    let mut tries = 0;
    loop {
      let v: FeatureEngineStatusReg = self.read(Reg::FeatureEngineStatus).await?;
      if v.axis_map_complete {
        return Ok(());
      }
      self.delay.delay_ms(2).await;
      tries += 1;
      if tries > 100 {
        return Err(Error::Data);
      }
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct FeatureEngineStatusReg {
  #[skip(10)]
  pub axis_map_complete: bool,
  #[bits(2)]
  pub engine_state: super::FeatureEngineState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AxisOrder {
  /// X => X, Y => Y, Z => Z
  XYZ = 0,
  /// X => Y, Y => X, Z => Z
  YXZ = 1,
  /// X => X, Y => Z, Z => Y
  XZY = 2,
  /// X => Z, Y => X, Z => Y
  ZXY = 3,
  /// X => Y, Y => Z, Z => X
  YZX = 4,
  /// X => Z, Y => Y, Z => X
  ZYX = 5,
}

impl From<AxisOrder> for u8 {
  fn from(value: AxisOrder) -> Self {
    value as u8
  }
}

impl core::convert::TryFrom<u8> for AxisOrder {
  type Error = ();
  fn try_from(v: u8) -> Result<Self, Self::Error> {
    match v {
      0 => Ok(AxisOrder::XYZ),
      1 => Ok(AxisOrder::YXZ),
      2 => Ok(AxisOrder::XZY),
      3 => Ok(AxisOrder::ZXY),
      4 => Ok(AxisOrder::YZX),
      5 => Ok(AxisOrder::ZYX),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct AxisRemap {
  #[bits(3)]
  pub order: AxisOrder,
  pub invert_x: bool,
  pub invert_y: bool,
  pub invert_z: bool,
}

impl Default for AxisRemap {
  fn default() -> Self {
    Self { order: AxisOrder::XYZ, invert_x: false, invert_y: false, invert_z: false }
  }
}

impl AxisRemap {
  pub const fn new(order: AxisOrder, invert_x: bool, invert_y: bool, invert_z: bool) -> Self {
    Self { order, invert_x, invert_y, invert_z }
  }
}
