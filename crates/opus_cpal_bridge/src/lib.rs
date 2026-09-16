use std::fmt::Debug;

use cpal::Host;

use crate::{
    error::StreamBuildError, macros::build_stream::build_input_stream, stream::{DeviceType, Stream}, stream_trait::StreamControls,
};

mod macros;
mod stream;

pub mod error;
pub mod stream_trait;

pub struct AudioBridge {
    host: Host,
    input_stream: Stream,
    output_stream: Stream,
}

impl AudioBridge {
    pub fn new() -> Result<Self, StreamBuildError> {
        let host = cpal::default_host();
        let mut input_stream = Stream::new(DeviceType::Input, &host)?;
        input_stream.play().expect("Failed to start input stream");
        let mut output_stream = Stream::new(DeviceType::Output, &host)?;
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

fn build_input_stream(stream: &mut Stream) {
    build_input_stream!(stream)
}

fn input_audio_callback<T>(buf: &[T], info: &cpal::InputCallbackInfo) {}

fn build_output_stream() {}

fn output_audio_callback<T>(buf: &mut [T], info: &cpal::OutputCallbackInfo) {}

fn error_callback(e: cpal::Error) {}
