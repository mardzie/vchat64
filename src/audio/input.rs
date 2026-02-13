use cpal::{
    Device, Host, PauseStreamError, PlayStreamError, Stream, SupportedStreamConfig,
    SupportedStreamConfigsError,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::audio::{
    config_filter::ConfigFilter,
    error::{DeviceType, StreamError},
};

pub struct InputStream {
    device: Device,
    config: SupportedStreamConfig,
    stream: Stream,
}

impl InputStream {
    pub fn new<T, D, E>(
        host: &Host,
        data_callback: D,
        error_callback: E,
    ) -> Result<Self, StreamError>
    where
        T: cpal::SizedSample,
        D: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::StreamError) + Send + 'static,
    {
        let device = host
            .default_input_device()
            .ok_or(StreamError::DefaultDeviceNotAvailable(DeviceType::Input))?;

        let supported_configs = match device.supported_input_configs() {
            Ok(supported_config) => supported_config,
            Err(e) => {
                return Err(match e {
                    SupportedStreamConfigsError::DeviceNotAvailable => {
                        StreamError::DeviceNotAvailable(DeviceType::Input)
                    }
                    SupportedStreamConfigsError::InvalidArgument => {
                        StreamError::InvalidArgument(DeviceType::Input)
                    }
                    SupportedStreamConfigsError::BackendSpecific { err } => {
                        StreamError::BackendSpecific(DeviceType::Input, err)
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
            .map_err(|e| StreamError::BuildStream(DeviceType::Input, e))?;

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
