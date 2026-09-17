use std::fmt::{Debug, Display};

use cpal::{
    Device, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{error::StreamBuildError, stream_trait::StreamControls};

pub struct Stream {
    device: Device,
    device_type: DeviceType,
    config: SupportedStreamConfig,
    /// `stream` contains the [`Stream`] and the playing indicator.
    inner: Option<(cpal::Stream, bool)>,
}

impl Stream {
    pub fn new(device_type: DeviceType, host: &cpal::Host) -> Result<Self, StreamBuildError> {
        let device = match device_type {
            DeviceType::Input => host.default_input_device(),
            DeviceType::Output => host.default_output_device(),
        }
        .ok_or(StreamBuildError::DefaultDeviceUnavailable(device_type))?;
        Self::from_device(device_type, device)
    }

    pub fn from_device(
        device_type: DeviceType,
        device: cpal::Device,
    ) -> Result<Self, StreamBuildError> {
        let config = Self::pick_config(device_type, &device)?;

        Ok(Self {
            device,
            device_type,
            config,
            inner: None,
        })
    }

    pub fn config(&self) -> SupportedStreamConfig {
        self.config
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    fn pick_config(
        device_type: DeviceType,
        device: &Device,
    ) -> Result<SupportedStreamConfig, cpal::Error> {
        let config_48k = match device_type {
            DeviceType::Input => Self::filter_config_48k(device.supported_input_configs()?),
            DeviceType::Output => Self::filter_config_48k(device.supported_output_configs()?),
        };
        let config = match config_48k {
            Some(config) => config,
            None => Self::default_config(device_type, device)?,
        };

        Ok(config)
    }

    fn default_config(
        device_type: DeviceType,
        device: &Device,
    ) -> Result<SupportedStreamConfig, cpal::Error> {
        match device_type {
            DeviceType::Input => device.default_input_config(),
            DeviceType::Output => device.default_output_config(),
        }
    }

    fn filter_config_48k(
        config_iter: impl IntoIterator<Item = cpal::SupportedStreamConfigRange>,
    ) -> Option<cpal::SupportedStreamConfig> {
        use cpal::{SAMPLE_RATE_48K, SampleFormat};

        config_iter
            .into_iter()
            .filter(|r| matches!(r.sample_format(), SampleFormat::F32 | SampleFormat::I16))
            .find(|r| {
                r.min_sample_rate() <= SAMPLE_RATE_48K && SAMPLE_RATE_48K <= r.max_sample_rate()
            })
            .map(|r| r.with_sample_rate(SAMPLE_RATE_48K))
    }

    pub fn build_input_stream<T, D, E>(
        &mut self,
        data_callback: D,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        D: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::Error) + Send + 'static,
    {
        let stream = self.device.build_input_stream(
            self.config.config(),
            data_callback,
            error_callback,
            None,
        )?;
        self.inner = Some((stream, false));

        Ok(())
    }

    pub fn build_output_stream<T, D, E>(
        &mut self,
        data_callback: D,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        D: FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static,
        E: FnMut(cpal::Error) + Send + 'static,
    {
        let stream = self.device.build_output_stream(
            self.config.config(),
            data_callback,
            error_callback,
            None,
        )?;
        self.inner = Some((stream, false));

        Ok(())
    }
}

impl StreamControls for Stream {
    fn play(&mut self) -> Result<(), crate::error::PlayPauseError> {
        if let Some((stream, playing)) = &mut self.inner {
            stream.play().map_err(crate::error::PlayPauseError::from)?;
            *playing = true;
        }

        Ok(())
    }

    fn pause(&mut self) -> Result<(), crate::error::PlayPauseError> {
        if let Some((stream, playing)) = &mut self.inner {
            stream.pause().map_err(crate::error::PlayPauseError::from)?;
            *playing = false;
        }

        Ok(())
    }

    fn playing(&self) -> bool {
        if let Some((_, playing)) = &self.inner {
            *playing
        } else {
            false
        }
    }
}

impl Debug for Stream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("device", &self.device)
            .field("device_type", &self.device_type)
            .field("config", &self.config)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
