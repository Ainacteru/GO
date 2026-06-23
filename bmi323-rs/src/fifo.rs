//! FIFO buffer configuration and data reading.
//!
//! The BMI323 includes a 2KB FIFO buffer that can store accelerometer,
//! gyroscope, temperature, and timestamp data.
//!
//! # Examples
//!
//! ```no_run
//! # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
//! use bmi323::fifo::FifoConfig;
//!
//! // Configure FIFO to store accelerometer and gyroscope data
//! let fifo_config = FifoConfig {
//!     acc_en: true,
//!     gyr_en: true,
//!     stop_on_full: false,
//!     ..Default::default()
//! };
//! imu.set_fifo_config(fifo_config).await.unwrap();
//!
//! // Set watermark level (triggers interrupt when reached)
//! imu.set_fifo_watermark(512).await.unwrap();
//!
//! // Read FIFO data
//! let mut buffer = [0u8; 1024];
//! let bytes_read = imu.read_fifo_bytes(&mut buffer).await.unwrap();
//! # }
//! ```

use embedded_hal_async::{delay::DelayNs, i2c::*};

use super::{defs::*, Bmi323, Error};

impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Configure FIFO producer sources and behavior.
  ///
  /// Context:
  /// - Enable only required producers (ACC/GYR/TEMP/TIME) to reduce bandwidth.
  /// - Use `stop_on_full` for lossless burst capture; otherwise oldest data
  ///   is overwritten when full.
  pub async fn set_fifo_config(&mut self, cfg: FifoConfig) -> Result<(), Error<E>> {
    self.write(Reg::FifoConf, cfg).await
  }

  /// Set FIFO watermark level (in words). Triggers watermark interrupt if enabled.
  ///
  /// Context:
  /// - Pick a watermark that balances interrupt rate vs latency/data age.
  pub async fn set_fifo_watermark(&mut self, level_words: u16) -> Result<(), Error<E>> {
    let mut r = FifoWatermark { watermark: 0 };
    r.watermark = core::cmp::min(level_words, (1u16 << 10) - 1);
    self.write(Reg::FifoWatermark, r).await
  }

  /// Read current FIFO fill level (in words).
  pub async fn get_fifo_fill_level(&mut self) -> Result<u16, Error<E>> {
    let r: FifoFillLevel = self.read(Reg::FifoFillLevel).await?;
    Ok(r.level)
  }

  /// Flush the FIFO content (configuration is preserved).
  pub async fn fifo_flush(&mut self) -> Result<(), Error<E>> {
    self.write(Reg::FifoCtrl, FifoCtrl { flush: true }).await
  }

  /// Read up to `out.len()` bytes from `FIFO_DATA`; returns bytes read.
  /// Uses current fill level to limit reads.
  ///
  /// Context:
  /// - For headless frames, consumers must know the enabled producers to parse
  ///   word order. Timestamp frames (TIME) help align to sensor time.
  pub async fn read_fifo_bytes(&mut self, out: &mut [u8]) -> Result<usize, Error<E>> {
    if out.is_empty() {
      return Ok(0);
    }
    let fill_words = self.get_fifo_fill_level().await? as usize;
    let available = fill_words.saturating_mul(2);
    let n = core::cmp::min(out.len(), available);
    if n == 0 {
      return Ok(0);
    }
    // Read contiguous bytes from FIFO_DATA.
    self.read_bytes(Reg::FifoData, &mut out[..n]).await?;
    Ok(n)
  }

  /// Read up to `out.len()` words (u16 LE) from `FIFO_DATA`; returns words read.
  pub async fn read_fifo_words(&mut self, out: &mut [u16]) -> Result<usize, Error<E>> {
    let max_bytes = out.len().saturating_mul(2);
    let mut tmp = [0u8; 256];
    let n = core::cmp::min(max_bytes, tmp.len());
    let nbytes = self.read_fifo_bytes(&mut tmp[..n]).await?;
    let mut w = 0usize;
    let mut i = 0usize;
    while i + 1 < nbytes && w < out.len() {
      out[w] = u16::from_le_bytes([tmp[i], tmp[i + 1]]);
      w += 1;
      i += 2;
    }
    Ok(w)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct FifoConfig {
  #[bits(1)]
  pub stop_on_full: bool,
  #[skip(7)]
  #[bits(1)]
  pub time_en: bool,
  #[bits(1)]
  pub accel_en: bool,
  #[bits(1)]
  pub gyro_en: bool,
  #[bits(1)]
  pub temp_en: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct FifoWatermark {
  #[bits(10)]
  pub watermark: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct FifoFillLevel {
  #[bits(11)]
  pub level: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
struct FifoCtrl {
  pub flush: bool,
}
