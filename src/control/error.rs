use core::fmt::Debug;

use thiserror::Error;

use crate::sensors::error::ImuError;

#[derive(Error, Debug)]
pub enum KalmanFilterError {
    #[error("ImuError Error")]
    ImuErr(ImuError),
}