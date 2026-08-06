use std::fmt::Debug;

use cpal::{
    Device, SAMPLE_RATE_48K, SampleFormat, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::error::{DeviceType, PlayPauseError, StreamBuildError};

pub struct InputStream {
    device: Device,
    config: SupportedStreamConfig,
    stream: Option<Stream>,
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

        Ok(Self {
            device,
            config,
            stream: None,
        })
    }

    pub fn build_stream<T, D, E>(
        &mut self,
        data_callback: D,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        D: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::Error) + Send + 'static,
    {
        self.stream = Some(self.device.build_input_stream(
            self.config.config(),
            data_callback,
            error_callback,
            None,
        )?);

        Ok(())
    }

    pub fn record(&mut self) -> Result<(), PlayPauseError> {
        if let Some(stream) = &self.stream {
            stream.play()?;
        }

        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), PlayPauseError> {
        if let Some(stream) = &self.stream {
            stream.pause()?;
        }

        Ok(())
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

impl Debug for InputStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputStream")
            .field("device", &self.device)
            .field("config", &self.config)
            .finish()
    }
}
