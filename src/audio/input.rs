use cpal::{
    Device, Host, PauseStreamError, PlayStreamError, Stream, SupportedStreamConfig,
    SupportedStreamConfigsError,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::audio::{
    config_filter::ConfigFilter,
    error::{DeviceType, Error},
};

pub struct InputStream {
    device: Device,
    config: SupportedStreamConfig,
    stream: Stream,
}

impl InputStream {
    pub fn new<T, D, E>(host: &Host, data_callback: D, error_callback: E) -> Result<Self, Error>
    where
        T: cpal::SizedSample,
        D: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::StreamError) + Send + 'static,
    {
        let device = host
            .default_input_device()
            .ok_or(Error::DefaultDeviceNotAvailable(DeviceType::Input))?;

        let supported_configs = match device.supported_input_configs() {
            Ok(supported_config) => supported_config,
            Err(e) => {
                return Err(match e {
                    SupportedStreamConfigsError::DeviceNotAvailable => {
                        Error::DeviceNotAvailable(DeviceType::Input)
                    }
                    SupportedStreamConfigsError::InvalidArgument => {
                        Error::InvalidArgument(DeviceType::Input)
                    }
                    SupportedStreamConfigsError::BackendSpecific { err } => {
                        Error::BackendSpecific(DeviceType::Input, err)
                    }
                });
            }
        };

        let config = ConfigFilter::from_supported_input_config(supported_configs)
            .filter_channel_count(2)
            .filter_sample_format(cpal::SampleFormat::F32)
            .pop()
            .expect("Failed to find acceptable input stream config.")
            .with_max_sample_rate();

        let stream = device
            .build_input_stream(&config.config(), data_callback, error_callback, None)
            .map_err(|e| Error::BuildStream(DeviceType::Input, e))?;

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
