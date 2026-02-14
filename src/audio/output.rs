use cpal::{
    Device, Host, PauseStreamError, PlayStreamError, Stream, SupportedStreamConfig,
    SupportedStreamConfigsError,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::audio::{
    config_filter::ConfigFilter,
    error::{DeviceType, Error},
};

pub struct OutputStream {
    device: Device,
    config: SupportedStreamConfig,
    stream: Stream,
}

impl OutputStream {
    pub fn new<T, D, E>(host: &Host, data_callback: D, error_callback: E) -> Result<Self, Error>
    where
        T: cpal::SizedSample,
        D: FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::StreamError) + Send + 'static,
    {
        let device = host
            .default_output_device()
            .ok_or(Error::DefaultDeviceNotAvailable(DeviceType::Output))?;

        let supported_configs = match device.supported_output_configs() {
            Ok(supported_config) => supported_config,
            Err(e) => {
                return Err(match e {
                    SupportedStreamConfigsError::DeviceNotAvailable => {
                        Error::DeviceNotAvailable(DeviceType::Output)
                    }
                    SupportedStreamConfigsError::InvalidArgument => {
                        Error::InvalidArgument(DeviceType::Output)
                    }
                    SupportedStreamConfigsError::BackendSpecific { err } => {
                        Error::BackendSpecific(DeviceType::Output, err)
                    }
                });
            }
        };

        let config = ConfigFilter::from_supported_output_config(supported_configs)
            .filter_channel_count(4)
            .filter_sample_format(cpal::SampleFormat::F32)
            .pop()
            .expect("Failed to find acceptable output stream config.")
            .with_max_sample_rate();

        let stream = device
            .build_output_stream(&config.config(), data_callback, error_callback, None)
            .map_err(|e| Error::BuildStream(DeviceType::Output, e))?;

        Ok(Self {
            device,
            config,
            stream,
        })
    }

    pub fn play(&self) -> Result<(), PlayStreamError> {
        self.stream.play()
    }

    pub fn pause(&self) -> Result<(), PauseStreamError> {
        self.stream.pause()
    }
}
