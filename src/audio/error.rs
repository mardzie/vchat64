use std::fmt::Display;

use cpal::{DefaultStreamConfigError, SupportedStreamConfigsError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

impl Error {
    pub fn from_supported_stream_configs_error(
        device_type: DeviceType,
        e: SupportedStreamConfigsError,
    ) -> Self {
        match e {
            SupportedStreamConfigsError::DeviceNotAvailable => {
                Error::DeviceNotAvailable(device_type)
            }
            SupportedStreamConfigsError::InvalidArgument => Error::InvalidArgument(device_type),
            SupportedStreamConfigsError::BackendSpecific { err } => {
                Error::BackendSpecific(device_type, err)
            }
        }
    }

    pub fn from_default_stream_config_error(
        device_type: DeviceType,
        e: DefaultStreamConfigError,
    ) -> Self {
        match e {
            DefaultStreamConfigError::DeviceNotAvailable => {
                Error::DefaultDeviceNotAvailable(device_type)
            }
            DefaultStreamConfigError::StreamTypeNotSupported => {
                panic!("Default input stream config not supported.")
            }
            DefaultStreamConfigError::BackendSpecific { err } => {
                Error::BackendSpecific(device_type, err)
            }
        }
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
        }
    }
}
