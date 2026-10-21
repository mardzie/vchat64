#[derive(Debug, thiserror::Error)]
pub enum AudioBridgeInitError {
    #[error(transparent)]
    StreamBuild(#[from] StreamBuildError),
    #[error(transparent)]
    Opus(#[from] opus::Error),
    #[error(transparent)]
    ResamplerConstruction(#[from] rubato::ResamplerConstructionError),
}

#[derive(Debug, thiserror::Error)]
pub enum StreamBuildError {
    #[error("default {0} device unavailable")]
    DefaultDeviceUnavailable(crate::stream::DeviceType),
    #[error("unsupported sample format {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),
    #[error(transparent)]
    Cpal(#[from] cpal::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PlayPauseError {
    #[error("device not available")]
    DeviceNotAvailable,
    #[error("stream invalidated")]
    StreamInvalidated,
    /// The Stream could not be paused.
    #[error("unsupported operation")]
    UnsupportedOperation,
}

impl From<cpal::Error> for PlayPauseError {
    fn from(e: cpal::Error) -> Self {
        use cpal::ErrorKind;

        match e.kind() {
            ErrorKind::DeviceNotAvailable => Self::DeviceNotAvailable,
            ErrorKind::StreamInvalidated => Self::StreamInvalidated,
            ErrorKind::UnsupportedOperation => Self::UnsupportedOperation,
            _ => unreachable!("{} is not a valid error for PlayPauseError", e),
        }
    }
}
