use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub enum StreamBuildError {
    #[error("Default {0} Device unavailable")]
    DefaultDeviceUnavailable(DeviceType),
    #[error("{0}")]
    Cpal(#[from] cpal::Error),
}

#[derive(Debug)]
pub enum DeviceType {
    Input,
    Output,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Input => write!(f, "input"),
            DeviceType::Output => write!(f, "output"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlayPauseError {
    #[error("Device not available")]
    DeviceNotAvailable,
    #[error("Stream invalidated")]
    StreamInvalidated,
}

impl From<cpal::Error> for PlayPauseError {
    fn from(e: cpal::Error) -> Self {
        use cpal::ErrorKind;

        match e.kind() {
            ErrorKind::DeviceNotAvailable => Self::DeviceNotAvailable,
            ErrorKind::StreamInvalidated => Self::StreamInvalidated,
            _ => unreachable!("{} is not a valid error for PlayPauseError", e),
        }
    }
}
