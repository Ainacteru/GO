#![no_std]
#![doc = include_str!("../README.md")]
//!
//! ## Design Principles
//!
//! - **Type-safe**: Strongly-typed configuration structs with sensible defaults
//! - **Async-first**: Built on `embedded-hal-async` I2C traits
//! - **Zero-copy**: Direct register access where possible
//! - **Documented**: Raw register fields include conversion formulas where applicable
//!
//! ## Module Organization
//!
//! - [`accel`]: Accelerometer configuration and data reading
//! - [`gyro`]: Gyroscope configuration and data reading
//! - [`fifo`]: FIFO buffer configuration and reading
//! - [`interrupt`]: Interrupt pin configuration and status
//! - [`feature`]: Feature engine for advanced motion detection
//! - [`calib`]: Calibration utilities
//! - [`selftest`]: Self-test functionality
//!
//! ## Basic Usage
//!
//! ```no_run
//! # async fn example() -> Result<(), bmi323::Error<()>> {
//! # use bmi323::{Bmi323, accel::AccelConfig};
//! # let i2c = (); // Your I2C implementation
//! # let delay = (); // Your delay implementation
//! let mut imu = Bmi323::new(i2c, delay);
//!
//! // Initialize and verify chip
//! imu.soft_reset().await?;
//! let chip_id = imu.get_id().await?;
//!
//! // Configure and read accelerometer
//! imu.set_accel_conf(AccelConfig::default()).await?;
//! let accel = imu.get_accel_data().await?;
//! # Ok(())
//! # }
//! ```

use embedded_hal_async::{delay::DelayNs, i2c::*};

pub mod accel;
pub mod alt;
pub mod calib;
mod defs;
#[cfg(feature = "events")]
mod events;
mod feature;
pub mod fifo;
pub mod gyro;
pub mod interrupt;
pub mod io;
pub mod offset;
pub(crate) mod rw;
pub mod selftest;
mod types;

use defs::*;
#[cfg(feature = "events")]
pub use events::*;
pub use feature::*;
pub use types::*;

/// Driver error type.
///
/// This error type wraps the underlying I2C error and adds BMI323-specific
/// error conditions.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E> {
  /// I2C communication error
  I2c(E),
  /// Chip ID mismatch (expected 0x43)
  InvalidChipId(u8),
  /// Invalid mode or configuration
  InvalidMode,
  /// Initialization failed (e.g., feature engine activation timeout)
  Init,
  /// Data error (e.g., timeout waiting for data ready)
  Data,
}

/// BMI323 device driver instance.
///
/// This is the main entry point for interacting with the BMI323 sensor.
/// It owns the I2C bus and delay provider, and maintains internal state
/// for the device.
///
/// # Type Parameters
///
/// - `I`: I2C implementation (must implement `embedded_hal_async::i2c::I2c`)
/// - `D`: Delay provider (must implement `embedded_hal_async::delay::DelayNs`)
/// - `W`: Interrupt wait implementation (only used with `events` feature)
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), bmi323::Error<()>> {
/// # use bmi323::Bmi323;
/// # let i2c = (); // Your I2C implementation
/// # let delay = (); // Your delay implementation
/// let mut imu = Bmi323::new(i2c, delay);
/// imu.soft_reset().await?;
/// # Ok(())
/// # }
/// ```
pub struct Bmi323<I, D: DelayNs, W = ()> {
  i2c: I,
  delay: D,
  accel_mult: f32,
  gyro_mult: f32,
  #[cfg(feature = "events")]
  dequeue: heapless::Deque<Event, 16>,
  #[cfg(feature = "events")]
  int_pin: W,
  #[cfg(not(feature = "events"))]
  _wait: core::marker::PhantomData<W>,
}

// Constructor(s)
#[cfg(feature = "events")]
impl<I, D, W> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress>,
  D: DelayNs,
  W: embedded_hal_async::digital::Wait,
{
  /// Create a new BMI323 driver instance with interrupt event support.
  ///
  /// # Arguments
  ///
  /// - `i2c`: I2C bus implementation
  /// - `delay`: Delay provider for timing operations
  /// - `int_pin`: Interrupt pin for event-driven operation (requires `events` feature)
  pub fn new(i2c: I, delay: D, int_pin: W) -> Self {
    Self { i2c, delay, accel_mult: 0.0, gyro_mult: 0.0, dequeue: heapless::Deque::new(), int_pin }
  }
}

#[cfg(not(feature = "events"))]
impl<I, D, W> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress>,
  D: DelayNs,
{
  /// Create a new BMI323 driver instance.
  ///
  /// # Arguments
  ///
  /// - `i2c`: I2C bus implementation
  /// - `delay`: Delay provider for timing operations
  pub fn new(i2c: I, delay: D) -> Self {
    Self { i2c, delay, accel_mult: 0.0, gyro_mult: 0.0, _wait: core::marker::PhantomData }
  }
}

// Common functionality (independent of `events`)
impl<I, D, W, E> Bmi323<I, D, W>
where
  I: I2c<SevenBitAddress, Error = E>,
  D: DelayNs,
{
  /// Read the chip ID register.
  ///
  /// Returns the chip ID (should be `0x43` for BMI323).
  /// Use this to verify communication with the sensor.
  ///
  /// # Example
  ///
  /// ```no_run
  /// # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
  /// let chip_id = imu.get_id().await.unwrap();
  /// assert_eq!(chip_id, 0x43);
  /// # }
  /// ```
  pub async fn get_id(&mut self) -> Result<u8, Error<E>> {
    let r: ChipId = self.read(Reg::ChipId).await?;
    Ok(r.id)
  }

  /// Perform a soft reset of the sensor.
  ///
  /// This resets all registers to their default values and restarts the sensor.
  /// A delay is automatically applied after the reset command.
  ///
  /// # Example
  ///
  /// ```no_run
  /// # async fn example(mut imu: bmi323::Bmi323<impl embedded_hal_async::i2c::I2c, impl embedded_hal_async::delay::DelayNs>) {
  /// imu.soft_reset().await.unwrap();
  /// # }
  /// ```
  pub async fn soft_reset(&mut self) -> Result<(), Error<E>> {
    self.write_u16(Reg::Cmd, Command::SoftReset.into()).await?;
    self.delay.delay_ms(SOFT_RESET_DELAY as u32).await;
    Ok(())
  }

  /// Read `ERR_REG` (raw bits per datasheet).
  pub async fn get_error(&mut self) -> Result<u16, Error<E>> {
    self.read_u16(Reg::Err).await
  }

  /// Wait until the selected sensor sets its data-ready bit.
  async fn wait_for(&mut self, sensor: Sensor) -> Result<(), Error<E>> {
    let mut retries = 0;
    while !self.is_ready(sensor).await? {
      if retries > 20 {
        return Err(Error::Data);
      }
      self.delay.delay_ms(2).await;
      retries += 1;
    }

    Ok(())
  }

  /// Check data-ready status for accel/gyro/temp using `STATUS` bits.
  async fn is_ready(&mut self, sensor: Sensor) -> Result<bool, Error<E>> {
    let st: Status = self.read(Reg::Status).await?;
    Ok(match sensor {
      Sensor::Accel => st.drdy_acc,
      Sensor::Gyro => st.drdy_gyr,
      Sensor::Temp => st.drdy_temp,
    })
  }

  // -----------------
  // Misc. data reads
  // -----------------
  /// Read raw temperature register (device-specific units).
  pub async fn get_temperature_raw(&mut self) -> Result<i16, Error<E>> {
    // TempData is 16-bit; convert to i16 LE.
    let raw: u16 = self.read_u16(Reg::TempData).await?;
    Ok(raw as i16)
  }

  /// Read 24-bit sensor time counter (wraps at 2^24). Units per datasheet.
  pub async fn get_sensor_time(&mut self) -> Result<u32, Error<E>> {
    let mut b = [0u8; 3];
    // SensorTime0..2 are consecutive starting at SensorTime0.
    self.read_bytes(Reg::SensorTime0, &mut b).await?;
    Ok((b[2] as u32) << 16 | (b[1] as u32) << 8 | (b[0] as u32))
  }

  /// Read saturation flags for accel/gyro axes.
  pub async fn get_saturation_flags(&mut self) -> Result<SaturationFlags, Error<E>> {
    self.read(Reg::SatFlags).await
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 1)]
struct Status {
  #[skip(5)]
  pub drdy_temp: bool,
  pub drdy_gyr: bool,
  pub drdy_acc: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 1)]
struct ChipId {
  pub id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[packbits::pack(bytes = 2)]
pub struct SaturationFlags {
  pub acc_x: bool,
  pub acc_y: bool,
  pub acc_z: bool,
  pub gyr_x: bool,
  pub gyr_y: bool,
  pub gyr_z: bool,
}
