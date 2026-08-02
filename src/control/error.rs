use core::fmt::Debug;

use thiserror::Error;

use crate::{sensors::error::ImuError, util::math::error::MatrixError};

#[derive(Error, Debug)]
pub enum KalmanFilterError {
    #[error("ImuError Error")]
    ImuErr(ImuError),
    #[error("Matrix Error")]
    Matrix(MatrixError),
}