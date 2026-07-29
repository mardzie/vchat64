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
