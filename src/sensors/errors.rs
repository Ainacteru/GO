#[derive(Debug)]
pub enum SensorError {
    NoId,
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::NoId => write!(f, "Could not retrieve the device's id"),
        }
    }
}