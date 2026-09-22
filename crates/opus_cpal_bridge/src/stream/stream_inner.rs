use std::{cmp::Reverse, fmt::Debug, marker::PhantomData};

use cpal::{
    Device, SAMPLE_RATE_48K, SampleFormat, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use crate::{error::StreamBuildError, stream::DeviceType};

mod sealed {
    pub trait Sealed {}
}

pub trait Direction: sealed::Sealed + 'static {
    const DEVICE_TYPE: DeviceType;

    type SupportedConfigs: Iterator<Item = cpal::SupportedStreamConfigRange>;

    fn default_device(host: &cpal::Host) -> Option<Device>;
    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error>;
    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input;
impl sealed::Sealed for Input {}
impl Direction for Input {
    const DEVICE_TYPE: DeviceType = DeviceType::Input;

    type SupportedConfigs = cpal::SupportedInputConfigs;

    fn default_device(host: &cpal::Host) -> Option<Device> {
        host.default_input_device()
    }

    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error> {
        device.supported_input_configs()
    }

    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error> {
        device.default_input_config()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Output;
impl sealed::Sealed for Output {}
impl Direction for Output {
    const DEVICE_TYPE: DeviceType = DeviceType::Output;

    type SupportedConfigs = cpal::SupportedOutputConfigs;

    fn default_device(host: &cpal::Host) -> Option<Device> {
        host.default_output_device()
    }

    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error> {
        device.supported_output_configs()
    }

    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error> {
        device.default_output_config()
    }
}

#[must_use]
pub struct StreamInner<D> {
    device: Device,
    config: SupportedStreamConfig,
    inner: Option<cpal::Stream>,

    _direction: PhantomData<D>,
}

impl<D: Direction> StreamInner<D> {
    pub fn new(host: &cpal::Host) -> Result<Self, StreamBuildError> {
        let device = D::default_device(host)
            .ok_or(StreamBuildError::DefaultDeviceUnavailable(D::DEVICE_TYPE))?;
        Self::from_device(device)
    }

    pub fn from_device(device: cpal::Device) -> Result<Self, StreamBuildError> {
        let config = Self::pick_config(&device)?;
        tracing::debug!("Stream config: {:?}", config);

        Ok(Self {
            device,
            config,
            inner: None,

            _direction: PhantomData,
        })
    }

    #[must_use]
    pub fn config(&self) -> SupportedStreamConfig {
        self.config
    }

    pub fn device_type(&self) -> DeviceType {
        D::DEVICE_TYPE
    }

    fn pick_config(device: &Device) -> Result<SupportedStreamConfig, cpal::Error> {
        let config = match Self::preferred_config_filter(D::supported_configs(device)?) {
            Some(config) => config,
            None => D::default_config(device)?,
        };
        Ok(config)
    }

    #[must_use]
    fn preferred_config_filter(
        config_iter: impl IntoIterator<Item = cpal::SupportedStreamConfigRange>,
    ) -> Option<cpal::SupportedStreamConfig> {
        config_iter
            .into_iter()
            .filter(|r| matches!(r.sample_format(), SampleFormat::F32))
            .filter_map(|r| r.try_with_sample_rate(SAMPLE_RATE_48K))
            .min_by_key(|r| r.channels())
    }

    fn build_inner_stream(&mut self, stream: cpal::Stream) -> Result<(), StreamBuildError> {
        stream.play()?;
        self.inner = Some(stream);

        Ok(())
    }
}

impl StreamInner<Input> {
    pub(crate) fn build_stream<T, F, E>(
        &mut self,
        data_callback: F,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        F: FnMut(&[T], &cpal::InputCallbackInfo) + Send + 'static,
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
}

impl StreamInner<Output> {
    pub(crate) fn build_stream<T, F, E>(
        &mut self,
        data_callback: F,
        error_callback: E,
    ) -> Result<(), StreamBuildError>
    where
        T: cpal::SizedSample,
        F: FnMut(&mut [T], &cpal::OutputCallbackInfo) + Send + 'static,
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
}

impl<D: Direction> Debug for StreamInner<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamInner")
            .field("device", &self.device)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}
