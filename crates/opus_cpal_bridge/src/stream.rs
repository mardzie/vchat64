use std::{
    cmp::Reverse,
    fmt::{Debug, Display},
};

use cpal::{
    Device, SAMPLE_RATE_48K, SampleFormat, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::error::StreamBuildError;

#[must_use]
pub struct Stream {
    device: Device,
    device_type: DeviceType,
    config: SupportedStreamConfig,
    /// `stream` contains the [`Stream`] and the playing indicator.
    inner: Option<cpal::Stream>,
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

    #[must_use]
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
        let config = match device_type {
            DeviceType::Input => Self::preferred_config_filter(device.supported_input_configs()?),
            DeviceType::Output => Self::preferred_config_filter(device.supported_output_configs()?),
        }
        .unwrap_or(Self::default_config(device_type, device)?);

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

    #[must_use]
    fn preferred_config_filter(
        config_iter: impl IntoIterator<Item = cpal::SupportedStreamConfigRange>,
    ) -> Option<cpal::SupportedStreamConfig> {
        fn rank(r: &cpal::SupportedStreamConfigRange) -> impl Ord + use<> {
            (
                r.channels(),
                Reverse(r.sample_format() == SampleFormat::F32),
            )
        }

        config_iter
            .into_iter()
            .filter(|r| matches!(r.sample_format(), SampleFormat::F32 | SampleFormat::I16))
            .filter(|r| {
                r.min_sample_rate() <= SAMPLE_RATE_48K && SAMPLE_RATE_48K <= r.max_sample_rate()
            })
            .min_by_key(rank)
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
        self.build_inner_stream(stream)
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
        self.build_inner_stream(stream)
    }

    fn build_inner_stream(&mut self, stream: cpal::Stream) -> Result<(), StreamBuildError> {
        stream.play()?;
        self.inner = Some(stream);

        Ok(())
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

#[must_use]
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
