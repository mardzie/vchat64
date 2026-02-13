use std::fmt::Display;

pub use cpal::{BackendSpecificError, DeviceId, DeviceIdError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Device not available: Retry with another device.")]
    DeviceNotAvailable(Result<DeviceId, DeviceIdError>),
    #[error("Backend Specific Error: {0}")]
    BackendSpecific(BackendSpecificError),
}

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Default {0} Device not available.")]
    DefaultDeviceNotAvailable(DeviceType),
    #[error("{0} Device not available")]
    DeviceNotAvailable(DeviceType),
    #[error("{0} Backend specific Error: {1}")]
    BackendSpecific(DeviceType, cpal::BackendSpecificError),
    #[error("Failed to build {0} Stream: {1}")]
    BuildStream(DeviceType, cpal::BuildStreamError),
    #[error("Invalid Argument in {0} Device")]
    InvalidArgument(DeviceType),
}

#[derive(Debug)]
pub enum DeviceType {
    Input,
    Output,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
        }
    }
}
