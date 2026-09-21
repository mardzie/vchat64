use std::{fmt::Display, ops::Deref, sync::Arc};

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

trait BufferDirection {
    type Buffer;
}

impl BufferDirection for Input {
    type Buffer = Consumer;
}

impl BufferDirection for Output {
    type Buffer = Producer;
}

pub struct Stream<D: BufferDirection> {
    inner: StreamInner<D>,
    ring_buf: D::Buffer,
}

impl<D: Direction + BufferDirection> Stream<D> {
    fn ringbuf_pair(ringbuf_size: usize) -> (Producer, Consumer) {
        ringbuf::SharedRb::new(ringbuf_size).split()
    }
}

impl Stream<Input> {
    pub fn new(host: &cpal::Host, ringbuf_size: usize) -> Result<Self, StreamBuildError> {
        let mut inner = StreamInner::<Input>::new(host)?;
        let (mut producer, consumer) = Self::ringbuf_pair(ringbuf_size);
        let config = inner.config();
        build_stream!(inner, audio_callback::input_audio_callback, (&mut producer, config.channels() as usize), {
            F32,
            F64,
            U8,
            U16,
            U32,
            U64,
            I8,
            I16,
            I32,
            I64
        });

        Ok(Self {
            inner,
            ring_buf: consumer,
        })
    }
}

impl Stream<Output> {
    pub fn new(host: &cpal::Host, ringbuf_size: usize) -> Result<Self, StreamBuildError> {
        let mut inner = StreamInner::<Output>::new(host)?;
        let (producer, mut consumer) = Self::ringbuf_pair(ringbuf_size);
        build_stream!(inner, audio_callback::output_audio_callback, (&mut consumer), {
            F32,
            F64,
            U8,
            U16,
            U32,
            U64,
            I8,
            I16,
            I32,
            I64
        });

        Ok(Self {
            inner,
            ring_buf: producer,
        })
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
    use cpal::Sample;
    use ringbuf::traits::{Consumer, Producer};

    pub fn input_audio_callback<T>(
        buf: &[T],
        _: &cpal::InputCallbackInfo,
        producer: &mut impl Producer<Item = f32>,
        channels: usize,
    ) where
        T: cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        // cpal guarantees that each frame has all channels.
        debug_assert_eq!(buf.len() % channels, 0);

        let inverse_channels = 1.0 / channels as f32;

        let mono_len = buf.len() / channels;
        let vacant_len = producer.vacant_len();
        let dropped = mono_len.saturating_sub(vacant_len);
        let buf_start = dropped * channels;
        let mono = buf[buf_start..].chunks_exact(channels).map(|frame| {
            // Average all channels into one mono frame per chunk.
            frame.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() * inverse_channels
        });
        let pushed = producer.push_iter(mono);
        let overrun = mono_len - pushed;

        debug_assert_eq!(overrun, dropped);
    }

    // TODO: Fill remaining space when not filled fully with silence.
    pub fn output_audio_callback<T>(
        buf: &mut [T],
        _: &cpal::OutputCallbackInfo,
        consumer: &mut impl Consumer<Item = f32>,
    ) where
        T: cpal::SizedSample + cpal::FromSample<f32>,
    {
        for (sample, new_sample) in buf.iter_mut().zip(consumer.pop_iter()) {
            *sample = new_sample.to_sample();
        }
    }
}
