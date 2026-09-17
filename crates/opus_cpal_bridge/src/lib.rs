use std::fmt::Debug;

use cpal::Host;
use ringbuf::{SharedRb, storage::Heap, traits::Split};

use crate::{
    error::StreamBuildError,
    macros::build_stream::{build_input_stream, build_output_stream},
    stream::{DeviceType, Stream},
    stream_trait::StreamControls,
};

mod macros;
mod stream;

pub mod error;
pub mod stream_trait;

pub const RINGBUF_SIZE: usize = 4 * 1024;

pub struct AudioBridge {
    host: Host,
    input_stream: Stream,
    output_stream: Stream,
}

impl AudioBridge {
    pub fn new() -> Result<Self, StreamBuildError> {
        let input_ringbuf: SharedRb<Heap<f32>> = ringbuf::SharedRb::new(RINGBUF_SIZE);
        let (mut input_producer, mut input_consumer) = input_ringbuf.split();
        let output_ringbuf: SharedRb<Heap<f32>> = ringbuf::SharedRb::new(RINGBUF_SIZE);
        let (mut output_producer, mut output_consumer) = output_ringbuf.split();

        let host = cpal::default_host();
        let mut input_stream = Stream::new(DeviceType::Input, &host)?;
        let input_config = input_stream.config();
        let channels = input_config.channels() as usize;
        build_input_stream!(input_stream, audio_callback::input_audio_callback, (&mut input_producer, channels), {
            F32,
            F64,
            U8,
            U16,
            U32,
            U64,
            I8,
            I16,
            I32,
            I64,
        });
        let mut output_stream = Stream::new(DeviceType::Output, &host)?;
        build_output_stream!(output_stream, audio_callback::output_audio_callback, (&mut output_consumer), {
            F32,
            F64,
            U8,
            U16,
            U32,
            U64,
            I8,
            I16,
            I32,
            I64,
        });

        input_stream.play().expect("Failed to start input stream");
        output_stream.play().expect("Failed to start output stream");

        Ok(Self {
            host,
            input_stream,
            output_stream,
        })
    }

    pub fn pop(&mut self) {}
}

impl Debug for AudioBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBridge").finish()
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
