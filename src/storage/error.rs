use core::{error::Error, fmt::{Debug, Display}};

#[derive(Debug)]
pub enum FlashError {
    OutOfBounds,
    RecordTooLarge,
}

impl Error for FlashError {}

impl Display for FlashError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutOfBounds => write!(f, "OutOfBounds"),
            Self::RecordTooLarge => write!(f, "RecordTooLarge"),
        }
    }
}