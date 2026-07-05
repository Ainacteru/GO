use core::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlashError {
    #[error("Message out of bounds")]
    OutOfBounds,
    #[error("Message too large")]
    RecordTooLarge,
}

#[derive(Error, Debug)]
  pub enum SDCardError {
      #[error("SD card timed out")]
      Timeout,
      #[error("No SD card")]
      NoSDCard,
      #[error("filesystem not mounted")]
      NotMounted,
      #[error("io error")]
      Io,
  }