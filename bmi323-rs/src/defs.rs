#![allow(dead_code)]

#[repr(u8)]
pub(crate) enum Reg {
  ChipId = 0x00,
  Err = 0x01,
  Status = 0x02,
  AccDataX = 0x03,
  GyrDataX = 0x06,
  TempData = 0x09,
  SensorTime0 = 0x0A,
  SatFlags = 0x0C,
  IntStatusInt1 = 0x0D,
  IntStatusInt2 = 0x0E,
  FeatureIo0 = 0x10,
  FeatureIo1 = 0x11,
  FeatureIo2 = 0x12,
  FeatureIo3 = 0x13,
  FeatureIoStatus = 0x14,
  FifoFillLevel = 0x15,
  FifoData = 0x16,
  AccConf = 0x20,
  GyrConf = 0x21,
  AltAccConf = 0x28,
  AltGyrConf = 0x29,
  AltConf = 0x2A,
  AltStatus = 0x2B,
  FifoWatermark = 0x35,
  FifoConf = 0x36,
  FifoCtrl = 0x37,
  IoIntCtrl = 0x38,
  IntConf = 0x39,
  IntMap1 = 0x3A,
  FeatureCtrl = 0x40,
  FeatureDataAddr = 0x41,
  FeatureDataTx = 0x42,
  FeatureDataStatus = 0x43,
  FeatureEngineStatus = 0x45,
  FeatureEventExt = 0x47,
  IoPdnCtrl = 0x4F,
  IoSpiIf = 0x50,
  IoPadStrength = 0x51,
  IoI2cIf = 0x52,
  IoOdrDeviation = 0x53,
  AccDpOffX = 0x60,
  AccDpDGainX = 0x61,
  AccDpOffY = 0x62,
  AccDpDGainY = 0x63,
  AccDpOffZ = 0x64,
  AccDpDGainZ = 0x65,
  GyrDpOffX = 0x66,
  GyrDpDGainX = 0x67,
  GyrDpOffY = 0x68,
  GyrDpDGainY = 0x69,
  GyrDpOffZ = 0x6A,
  GyrDpDGainZ = 0x6B,
  I3cTcSyncTph = 0x70,
  I3cTcSyncTu = 0x71,
  I3cTcSyncOdr = 0x72,
  Cmd = 0x7E,
  CfgRes = 0x7F,
}

#[repr(u16)]
pub(crate) enum Command {
  SelfTestTrigger = 0x0100,
  AxisMapUpdate = 0x0300,
  SoftReset = 0xDEAF,
}

impl From<Command> for u16 {
  #[inline]
  fn from(c: Command) -> Self {
    c as u16
  }
}

// Constants used across the crate
pub(crate) const BMI323_CHIP_ID: u8 = 0x43;
pub(crate) const SOFT_RESET_DELAY: u16 = 1500; // us per datasheet

// I2C address (primary)
pub(crate) const ADDR_I2C_PRIM: u8 = 0x68;
