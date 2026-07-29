use cpal::{
    Device, SAMPLE_RATE_48K, SampleFormat, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait},
};

use crate::error::{DeviceType, StreamBuildError};

#[derive(Debug)]
pub struct InputStream {
    device: Device,
}

impl InputStream {
    pub fn new(host: &cpal::Host) -> Result<Self, StreamBuildError> {
        let device =
            host.default_input_device()
                .ok_or(StreamBuildError::DefaultDeviceUnavailable(
                    DeviceType::Input,
                ))?;
        let config = Self::pick_input_config(&device)?;

        tracing::info!("Input stream created");
    }

    fn pick_input_config(device: &Device) -> Result<SupportedStreamConfig, cpal::Error> {
        let config_48k = device
            .supported_input_configs()?
            .into_iter()
            .filter(|r| matches!(r.sample_format(), SampleFormat::F32 | SampleFormat::I16))
            .find(|r| {
                r.min_sample_rate() <= SAMPLE_RATE_48K && SAMPLE_RATE_48K <= r.max_sample_rate()
            })
            .map(|r| r.with_sample_rate(SAMPLE_RATE_48K));
        match config_48k {
            Some(cfg) => Ok(cfg),
            None => Ok(device.default_input_config()?),
        }
    }
}
