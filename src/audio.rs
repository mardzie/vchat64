use std::sync::mpsc::{Receiver, Sender};

use cpal::{Host, InputCallbackInfo, OutputCallbackInfo, default_host};

use crate::audio::{audio_processor::AudioProcessor, input::InputStream, output::OutputStream};

pub mod audio_processor;
pub mod config_filter;
pub mod error;
pub mod input;
pub mod output;

pub struct Audio {
    host: Host,
    input: InputStream,
    output: OutputStream,
    audio_processor: AudioProcessor,
}

impl Audio {
    /// Creates a new [`Audio`] instance.
    ///
    /// `input_channel` is the channel where the microphone sends its data.
    ///
    /// `output_channel` is the channel where the data the speaker should play gets sent.
    pub fn new(input_channel: Sender<Vec<f32>>, output_channel: Receiver<Vec<f32>>) -> Self {
        let host = default_host();

        let input = InputStream::new(
            &host,
            move |buf: &[f32], info| Self::input_data_callback(buf, info, &input_channel),
            move |e| log::error!("Input Stream Error: {}", e),
        )
        .expect("Failed to create new input stream.");

        let output = OutputStream::new(
            &host,
            move |buf, info| Self::output_data_callback(buf, info, &output_channel),
            move |e| log::error!("Output Stream Error: {}", e),
        )
        .expect("Failed to create new output stream.");

        Self {
            input,
            output,
            audio_processor: AudioProcessor {},
            host,
        }
    }

    fn input_data_callback(
        buf: &[f32],
        info: &InputCallbackInfo,
        input_channel: &Sender<Vec<f32>>,
    ) {
        todo!("Process Audio and ship it to `input_channel`");
    }

    fn output_data_callback(
        buf: &mut [f32],
        info: &OutputCallbackInfo,
        output_channel: &Receiver<Vec<f32>>,
    ) {
        let values = loop {
            match output_channel.recv() {
                Ok(values) => break values,
                Err(e) => {
                    log::error!("Sender of audio output closed: {}", e);
                    return;
                }
            }
        };

        let len = values.len().min(buf.len());
        buf[..len].copy_from_slice(&values[..len]);
    }

    pub fn play(&self) {
        self.play_input();
        self.play_output();
    }

    pub fn play_input(&self) {
        self.input.play().expect("Failed to play input stream.");
    }

    pub fn play_output(&self) {
        self.output.play().expect("Failed to play output stream.");
    }

    pub fn pause(&self) {
        self.pause_input();
        self.pause_output();
    }

    pub fn pause_input(&self) {
        self.input.pause().expect("Failed to pause input stream.");
    }

    pub fn pause_output(&self) {
        self.output.pause().expect("Failed to pause output stream.");
    }
}
