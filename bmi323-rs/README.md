# Bosch BMI323 IMU

[![crates.io](https://img.shields.io/crates/v/bmi323-rs.svg)](https://crates.io/crates/bmi323-rs)
[![Documentation](https://docs.rs/bmi323-rs/badge.svg)](https://docs.rs/bmi323-rs)

A `no_std`, async Rust driver for the [Bosch BMI323](https://www.bosch-sensortec.com/products/motion-sensors/imus/bmi323/) 6-axis IMU sensor using `embedded-hal-async` I2C traits.

## Features

- 🚀 **Async I2C** via `embedded-hal-async`
- 📦 **no_std** compatible
- 🎯 **Type-safe** configuration with strongly-typed enums and bitfields
- 🔧 **Comprehensive API** for accelerometer, gyroscope, and advanced features
- 📊 **FIFO support** with configurable watermarks
- 🔔 **Interrupt handling** with flexible mapping
- 🎨 **Feature engine** for motion detection, tap, step counting, orientation, and more
- 📝 **Optional defmt** logging support

## Quick Start

```rust
use bmi323::{Bmi323, accel::AccelConfig, gyro::GyroConfig};

// Create the driver
let mut imu = Bmi323::new(i2c, delay);

// Initialize
imu.soft_reset().await?;
let chip_id = imu.get_id().await?;

// Configure accelerometer
let accel_config = AccelConfig::default();
imu.set_accel_conf(accel_config).await?;

// Read accelerometer data
let accel_data = imu.get_accel_data().await?; // Returns Vector3d<f32> in g
println!("Accel: x={}, y={}, z={}", accel_data.x, accel_data.y, accel_data.z);

// Configure gyroscope
let gyro_config = GyroConfig::default();
imu.set_gyro_conf(gyro_config).await?;

// Read gyroscope data
let gyro_data = imu.get_gyro_data().await?; // Returns Vector3d<f32> in dps
```

## Advanced Features

### Feature Engine

The BMI323 includes an on-chip feature engine for advanced motion detection:

```rust
// Enable the feature engine
imu.enable_feature_engine().await?;

// Configure tap detection
use bmi323::feature::tap::*;
let tap_config = TapConfig::default();
imu.set_tap_config(tap_config).await?;

// Configure step counter
use bmi323::feature::step::*;
let step_config = StepCounterConfig::default();
imu.set_step_counter_config(step_config).await?;
```

### FIFO

```rust
use bmi323::fifo::FifoConfig;

let fifo_config = FifoConfig {
    acc_en: true,
    gyr_en: true,
    stop_on_full: false,
    ..Default::default()
};
imu.set_fifo_config(fifo_config).await?;
imu.set_fifo_watermark(512).await?;

// Read FIFO data
let mut buffer = [0u8; 1024];
let bytes_read = imu.read_fifo_bytes(&mut buffer).await?;
```

### Interrupts

```rust
use bmi323::interrupt::*;

// Configure interrupt pins
let int1 = IntConfig {
    level: IntLevel::ActiveHigh,
    output: IntOutputMode::PushPull,
    enable: true,
};
imu.set_int_config(true, int1, IntConfig::default()).await?;

// Map features to interrupt pins
let int_map = IntMap {
    any_motion: IntPin::Int1,
    tap: IntPin::Int1,
    ..Default::default()
};
imu.set_int_map(int_map).await?;
```

## Cargo Features

- `defmt`: Enable defmt logging support for debugging
- `events`: Enable interrupt event processing with internal queue

## Hardware Support

This driver supports the BMI323 IMU via I2C. The BMI323 features:

- 16-bit accelerometer with ±2g to ±16g ranges
- 16-bit gyroscope with ±125°/s to ±2000°/s ranges
- Programmable output data rates from 0.78 Hz to 6.4 kHz
- 2KB FIFO buffer
- Advanced motion features (tap, step, orientation, tilt, etc.)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
