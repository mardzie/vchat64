use std::{fmt::Debug, sync::Arc};

use cpal::{Host, Sample};
use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Consumer, Producer, Split},
    wrap::caching::Caching,
};

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

pub const RINGBUF_SIZE: usize = 16 * 1024;

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
        build_input_stream!(input_stream, input_audio_callback, (&mut input_producer), {
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
        build_output_stream!(output_stream, output_audio_callback, (&mut output_consumer), {
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
}

impl Debug for AudioBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBridge").finish()
    }
}

#[inline(always)]
fn input_audio_callback<T>(
    buf: &[T],
    _: &cpal::InputCallbackInfo,
    producer: &mut Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
) where
    T: Copy + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    producer.push_iter(buf.iter().map(|sample| sample.to_sample::<f32>()));
}

// TODO: Fill remaining space when not filled fully with silence.
#[inline(always)]
fn output_audio_callback<T>(
    buf: &mut [T],
    _: &cpal::OutputCallbackInfo,
    consumer: &mut Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
) where
    T: Copy + cpal::FromSample<f32>,
    f32: cpal::SizedSample,
{
    for (sample, new_sample) in buf.iter_mut().zip(consumer.pop_iter()) {
        *sample = new_sample.to_sample();
    }
}
