use core::fmt::Debug;

use thiserror::Error;

use crate::{sensors::error::{BarometerError, ImuError}, util::math::error::MatrixError};

#[derive(Error, Debug)]
pub enum KalmanFilterError {
    #[error("Imu Error")]
    ImuErr(ImuError),
    #[error("Barometer Error")]
    BarometerErr(BarometerError),
    #[error("Matrix Error")]
    Matrix(MatrixError),
}