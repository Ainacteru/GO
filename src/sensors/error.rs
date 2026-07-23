use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImuError {
    #[error("Power Error")]
    Power,
    #[error("Initialization Error")]
    Initialization,
    #[error("Accelerometer Read Error")]
    AccelRead,
    #[error("Gyroscope Read Error")]
    GyroRead,
}