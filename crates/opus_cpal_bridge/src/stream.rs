use std::{
    fmt::{Debug, Display},
    num::NonZero,
    ops::Deref,
    sync::Arc,
};

use cpal::ChannelCount;
use ringbuf::{SharedRb, storage::Heap, traits::Split, wrap::caching::Caching};

use crate::{
    error::StreamBuildError,
    macros::build_stream::build_stream,
    stream::stream_inner::{Direction, Input, Output, StreamInner},
};

pub mod stream_inner;

type InnerRb = Arc<SharedRb<Heap<f32>>>;
type Producer = Caching<InnerRb, true, false>;
type Consumer = Caching<InnerRb, false, true>;

mod sealed {
    pub trait Sealed {}
}

pub trait BufferDirection: sealed::Sealed {
    type Buffer;
}

impl BufferDirection for Input {
    type Buffer = Consumer;
}
impl sealed::Sealed for Input {}

impl BufferDirection for Output {
    type Buffer = Producer;
}
impl sealed::Sealed for Output {}

pub struct Stream<D> {
    inner: StreamInner<D>,
}

impl<D> Stream<D> {
    fn ringbuf_pair(ringbuf_size: usize) -> (Producer, Consumer) {
        ringbuf::SharedRb::new(ringbuf_size).split()
    }

    fn channels(inner: &StreamInner<D>) -> NonZero<ChannelCount> {
        let config = inner.config();
        NonZero::new(config.channels())
            .expect("CPAL reported an invalid zero-channel configuration")
    }
}

impl Stream<Input> {
    pub fn new(
        host: &cpal::Host,
        ringbuf_size: usize,
    ) -> Result<(Self, <Input as BufferDirection>::Buffer), StreamBuildError> {
        let mut inner = StreamInner::<Input>::new(host)?;
        let (mut producer, consumer) = Self::ringbuf_pair(ringbuf_size);
        let channels = Self::channels(&inner);
        build_stream!(inner, audio_callback::input_audio_callback, (&mut producer, channels), {
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
            I64
        });

        Ok((Self { inner }, consumer))
    }
}

impl Stream<Output> {
    pub fn new(
        host: &cpal::Host,
        ringbuf_size: usize,
    ) -> Result<(Self, <Output as BufferDirection>::Buffer), StreamBuildError> {
        let mut inner = StreamInner::<Output>::new(host)?;
        let (producer, mut consumer) = Self::ringbuf_pair(ringbuf_size);
        let channels = Self::channels(&inner);
        build_stream!(inner, audio_callback::output_audio_callback, (&mut consumer, channels), {
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
            I64
        });

        Ok((Self { inner }, producer))
    }
}

impl<D> Deref for Stream<D> {
    type Target = StreamInner<D>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D: Direction> Debug for Stream<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
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

mod audio_callback {
    use std::num::NonZero;

    use cpal::{ChannelCount, Sample};
    use ringbuf::traits::{Consumer, Producer};

    pub fn input_audio_callback<T>(
        buf: &[T],
        _: &cpal::InputCallbackInfo,
        producer: &mut impl Producer<Item = f32>,
        channels: NonZero<ChannelCount>,
    ) where
        T: cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        // cpal guarantees that each frame has all channels.
        debug_assert_eq!(buf.len() % channels.get() as usize, 0);

        let inverse_channels = 1.0 / channels.get() as f32;

        let mono_len = buf.len() / channels;
        let vacant_len = producer.vacant_len();
        let dropped = mono_len.saturating_sub(vacant_len);
        let buf_start = dropped * channels;
        let mono = buf[buf_start..]
            .chunks_exact(channels.get() as usize)
            .map(|frame| {
                // Average all channels into one mono frame per chunk.
                frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() * inverse_channels
            });
        let pushed = producer.push_iter(mono);
        let overrun = mono_len - pushed;

        debug_assert_eq!(overrun, dropped);
    }

    pub fn output_audio_callback<T>(
        buf: &mut [T],
        _: &cpal::OutputCallbackInfo,
        consumer: &mut impl Consumer<Item = f32>,
        channels: NonZero<ChannelCount>,
    ) where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        // cpal guarantees that each frame has all channels.
        debug_assert_eq!(buf.len() % channels.get() as usize, 0);

        // Silence buffer
        buf.fill(T::EQUILIBRIUM);

        for (frame, sample) in buf
            .chunks_exact_mut(channels.get() as usize)
            .zip(consumer.pop_iter())
        {
            frame.fill(sample.to_sample());
        }
    }
}
