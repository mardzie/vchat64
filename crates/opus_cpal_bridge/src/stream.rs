use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    num::NonZero,
};

use cpal::{
    ChannelCount, Device, SAMPLE_RATE_48K, SampleFormat, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::{HeapCons, HeapProd, HeapRb, traits::Split};

use crate::{FRAME, error::StreamBuildError, macros::build_stream::build_stream};

mod sealed {
    pub trait Sealed {}
}

/// The Direction of a [`Stream<D>`].
pub trait Direction: sealed::Sealed + 'static {
    const DEVICE_TYPE: DeviceType;

    type SupportedConfigs: Iterator<Item = cpal::SupportedStreamConfigRange>;
    /// Ring half returned to the caller: HeapCons<f32> for Input, HeapProd<f32> for Output.
    type Handle;

    fn default_device(host: &cpal::Host) -> Option<Device>;
    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error>;
    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error>;
    fn build(
        device: &Device,
        config: SupportedStreamConfig,
        rb: HeapRb<f32>,
    ) -> Result<(cpal::Stream, Self::Handle), StreamBuildError>;
}

/// Input device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input;
impl sealed::Sealed for Input {}
impl Direction for Input {
    const DEVICE_TYPE: DeviceType = DeviceType::Input;

    type SupportedConfigs = cpal::SupportedInputConfigs;
    type Handle = HeapCons<f32>;

    fn default_device(host: &cpal::Host) -> Option<Device> {
        host.default_input_device()
    }

    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error> {
        device.supported_input_configs()
    }

    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error> {
        device.default_input_config()
    }

    fn build(
        device: &Device,
        config: SupportedStreamConfig,
        rb: HeapRb<f32>,
    ) -> Result<(cpal::Stream, Self::Handle), StreamBuildError> {
        let (mut producer, consumer) = rb.split();
        let channels = non_zero_channels(&config);
        let stream = build_stream!(
            device,
            build_input_stream,
            config,
            audio_callback::input_audio_callback,
            (&mut producer, channels),
            {
                F32,
                F64,
                U8,
                U16,
                U24,
                U32,
                U64,
                I8,
                I16,
                I24,
                I32,
                I64,
            }
        )?;
        Ok((stream, consumer))
    }
}

/// Output device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Output;
impl sealed::Sealed for Output {}
impl Direction for Output {
    const DEVICE_TYPE: DeviceType = DeviceType::Output;

    type SupportedConfigs = cpal::SupportedOutputConfigs;
    type Handle = HeapProd<f32>;

    fn default_device(host: &cpal::Host) -> Option<Device> {
        host.default_output_device()
    }

    fn supported_configs(device: &Device) -> Result<Self::SupportedConfigs, cpal::Error> {
        device.supported_output_configs()
    }

    fn default_config(device: &Device) -> Result<cpal::SupportedStreamConfig, cpal::Error> {
        device.default_output_config()
    }

    fn build(
        device: &Device,
        config: SupportedStreamConfig,
        rb: HeapRb<f32>,
    ) -> Result<(cpal::Stream, Self::Handle), StreamBuildError> {
        let (producer, mut consumer) = rb.split();
        let channels = non_zero_channels(&config);
        let stream = build_stream!(
            device,
            build_output_stream,
            config,
            audio_callback::output_audio_callback,
            (&mut consumer, channels),
            {
                F32,
                F64,
                U8,
                U16,
                U24,
                U32,
                U64,
                I8,
                I16,
                I24,
                I32,
                I64,
            }
        )?;
        Ok((stream, producer))
    }
}

/// A cpal Stream wrapper.
///
/// Works with mono `f32` (-1.0 <= s < 1.0) audio samples.
///
/// The Stream will play as soon as it is constructed.
///
/// # [`Input`]
/// A `Stream<Input>` collects samples from the input device into the ring.
/// The ring needs to be emptied regularly.
/// If the ring is at capacity, new samples from the output device will be dropped.
/// Only the freshest samples will be written into the ring if the rings free space is insufficient.
/// Older samples will be dropped.
/// The devices [`SampleFormat`] will be converted to `f32`.
/// All channels will be averaged to mono audio.
///
/// # [`Output`]
/// A `Stream<Output>` writes samples from the ring into the output devices buffer.
/// The ring needs to be filled regularly.
/// If there aren't enough samples in the ring to satisfy the output devices buffer, the audio wont behave as expected.
/// The `f32` samples from the ring will be converted to the devices [`SampleFormat`].
/// The ring samples will be written to all channels.
pub struct Stream<D> {
    device: Device,
    config: SupportedStreamConfig,
    /// Held for its `Drop`. Dropping stops the callback.
    stream: cpal::Stream,
    _direction: PhantomData<D>,
}

impl<D: Direction> Stream<D> {
    /// New default device from `host` and a ring buffer capacity of `capacity`.
    pub fn new(host: &cpal::Host, capacity: usize) -> Result<(Self, D::Handle), StreamBuildError> {
        let device = D::default_device(host)
            .ok_or(StreamBuildError::DefaultDeviceUnavailable(D::DEVICE_TYPE))?;
        Self::from_device(device, capacity)
    }

    /// From a device with a ring buffer capacity of `capacity`.
    pub fn from_device(
        device: Device,
        capacity: usize,
    ) -> Result<(Self, D::Handle), StreamBuildError> {
        let config = pick_config::<D>(&device)?;
        tracing::debug!("{} Stream config: {:?}", D::DEVICE_TYPE, config);

        let (stream, handle) = D::build(&device, config, HeapRb::new(capacity))?;
        let this = Self {
            device,
            config,
            stream,
            _direction: PhantomData,
        };
        this.warn_if_ring_undersized(capacity);
        this.stream.play()?;

        Ok((this, handle))
    }

    /// The configuration of the underlying cpal stream.
    pub fn config(&self) -> SupportedStreamConfig {
        self.config
    }

    /// The [`DeviceType`].
    pub fn device_type(&self) -> DeviceType {
        D::DEVICE_TYPE
    }

    /// Output warning if the provided buffer size is smaller than the recommended.
    fn warn_if_ring_undersized(&self, capacity: usize) {
        if capacity < FRAME {
            panic!(
                "`capacity({})` can not be smaller than FRAME: {}",
                capacity, FRAME
            )
        };

        match self.stream.buffer_size() {
            Ok(size) => {
                if capacity < 2 * size as usize {
                    tracing::warn!(
                        "Provided ring buffer size is smaller than double the estimated buffer size: (current: {}; 2x estimate: {})",
                        capacity,
                        2 * size
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to get buffer size estimate for ring buffer: {}", e)
            }
        }
    }
}

impl<D: Direction> Debug for Stream<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("device", &self.device)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Input,
    Output,
}

impl From<Input> for DeviceType {
    fn from(_: Input) -> Self {
        Self::Input
    }
}

impl From<Output> for DeviceType {
    fn from(_: Output) -> Self {
        Self::Output
    }
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Input => write!(f, "Input"),
            DeviceType::Output => write!(f, "Output"),
        }
    }
}

fn pick_config<D: Direction>(device: &Device) -> Result<SupportedStreamConfig, cpal::Error> {
    let config = match preferred_config_filter(D::supported_configs(device)?) {
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

fn non_zero_channels(config: &cpal::SupportedStreamConfig) -> NonZero<ChannelCount> {
    NonZero::new(config.channels()).expect("CPAL reported an invalid zero-channel configuration")
}

mod audio_callback {
    use std::num::NonZero;

    use cpal::{ChannelCount, Sample};
    use ringbuf::{
        HeapCons, HeapProd,
        traits::{Consumer, Observer, Producer},
    };

    pub fn input_audio_callback<T>(
        buf: &[T],
        _: &cpal::InputCallbackInfo,
        producer: &mut HeapProd<f32>,
        channels: NonZero<ChannelCount>,
    ) where
        T: cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        // cpal guarantees that each frame has all channels.
        debug_assert_eq!(buf.len() % channels.get() as usize, 0);

        let channels = channels.get() as usize;
        let inverse_channels = 1.0 / channels as f32;

        let mono_len = buf.len() / channels;
        let vacant_len = producer.vacant_len();
        // Drop the oldest samples.
        let dropped = mono_len.saturating_sub(vacant_len);
        let buf_start = dropped * channels;
        let mono = buf[buf_start..].chunks_exact(channels).map(|frame| {
            // Average all channels into one mono frame per chunk.
            frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() * inverse_channels
        });
        let pushed = producer.push_iter(mono);
        let overrun = mono_len.saturating_sub(pushed);

        // The overrun are samples that could not be written to the producer.
        // The space of the producer should never decrease without writing to it.
        // The oldest samples that dont fit get dropped first.
        // If then the overrun isnt the same as the dropped then the producer had not as much space as advertised.
        debug_assert_eq!(overrun, dropped);
    }

    pub fn output_audio_callback<T>(
        // Buffer already comes silenced from cpal >= 0.18. Just write in it.
        buf: &mut [T],
        _: &cpal::OutputCallbackInfo,
        consumer: &mut HeapCons<f32>,
        channels: NonZero<ChannelCount>,
    ) where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        // cpal guarantees that each frame has all channels.
        debug_assert_eq!(buf.len() % channels.get() as usize, 0);

        for (frame, sample) in buf.chunks_exact_mut(channels.get() as usize).zip(
            consumer
                .pop_iter()
                .map(|raw_sample| raw_sample.clamp(-1.0, 1.0 - f32::EPSILON / 2.0)),
        ) {
            frame.fill(sample.to_sample());
        }
    }
}
