use core::{array::TryFromSliceError, fmt::Debug};

use embassy_embedded_hal::shared_bus::I2cDeviceError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImuError {
    #[error("Power Error")]
    Power,
    #[error("Initialization Error")]
    Initialization,
    #[error("Accelerometer Read Error")]
    AccelRead,
    #[error("Accelerometer Config Error")]
    AccelConfig,
    #[error("Gyroscope Read Error")]
    GyroRead,
    #[error("Gyroscope Config Error")]
    GyroConfig,
    #[error("Soft Reset Error")]
    SoftReset,
    #[error("I2C Error")]
    I2C,
}

#[derive(Error, Debug)]
pub enum BarometerError{
    #[error("Power Error")]
    Power,
    #[error("Initialization Error")]
    Initialization,
    #[error("Temperature Read Error")]
    TempRead,
    #[error("Soft Reset Error")]
    SoftReset,
    #[error("I2C Error")]
    I2C,
    #[error("Array Error")]
    Array(TryFromSliceError)
}