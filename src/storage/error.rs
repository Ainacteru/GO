use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("Message out of bounds")]
    OutOfBounds,
    #[error("Message too large")]
    RecordTooLarge,
    #[error("Message not valid UTF-8")]
    BadUTF8,
}