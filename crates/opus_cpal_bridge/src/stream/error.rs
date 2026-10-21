#[derive(thiserror::Error, Debug)]
pub enum StreamBuildError {
    #[error("default {0} device unavailable")]
    DefaultDeviceUnavailable(crate::stream::DeviceType),
    #[error("unsupported sample format {0}")]
    UnsupportedSampleFormat(cpal::SampleFormat),
    #[error(transparent)]
    Cpal(#[from] cpal::Error),
}
