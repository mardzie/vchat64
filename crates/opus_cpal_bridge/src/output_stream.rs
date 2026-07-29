use cpal::traits::HostTrait;

use crate::error::{DeviceType, StreamBuildError};

#[derive(Debug)]
pub struct OutputStream {}

impl OutputStream {
    pub fn new(host: &cpal::Host) -> Result<Self, StreamBuildError> {
        let device =
            host.default_output_device()
                .ok_or(StreamBuildError::DefaultDeviceUnavailable(
                    DeviceType::Output,
                ))?;
    }
}
