use std::sync::{
    Arc, atomic,
    mpsc::{Receiver, Sender},
};

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

    volume: Arc<atomic::AtomicU8>,
    cutoff: Arc<atomic::AtomicU8>,
}

impl Audio {
    /// Creates a new [`Audio`] instance.
    ///
    /// `input_channel` is the channel where the microphone data gets sent after processing.
    ///
    /// `output_channel` is the channel that connects to the speaker.
    pub fn new(
        input_channel: Sender<Vec<f32>>,
        output_channel: Receiver<Vec<f32>>,
        init_volume: u8,
        init_cutoff: u8,
    ) -> Self {
        let host = default_host();

        let (input_tx_to_audio_processor, audio_processor_rx) = std::sync::mpsc::channel();

        let input = InputStream::new(
            &host,
            move |buf, info| Self::input_data_callback(buf, info, &input_tx_to_audio_processor),
            move |e| log::error!("Input Stream Error: {}", e),
        )
        .expect("Failed to create new input stream.");

        let output = OutputStream::new(
            &host,
            move |buf, info| Self::output_data_callback(buf, info, &output_channel),
            move |e| log::error!("Output Stream Error: {}", e),
        )
        .expect("Failed to create new output stream.");

        let volume = Arc::new(atomic::AtomicU8::new(init_volume));
        let cutoff = Arc::new(atomic::AtomicU8::new(init_cutoff));
        
        let volume_c = volume.clone();
        let cutoff_c = cutoff.clone();
        let audio_processor =
            AudioProcessor::new(audio_processor_rx, input_channel, volume_c, cutoff_c);

        Self {
            input,
            output,
            audio_processor,
            host,

            volume,
            cutoff,
        }
    }

    fn input_data_callback(
        buf: &[f32],
        info: &InputCallbackInfo,
        input_channel: &Sender<Vec<f32>>,
    ) {
        let buf = buf.to_vec();

        match input_channel.send(buf) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Input Device: Receiver closed channel stopping input device: {}",
                    e
                )
            }
        }
    }

    fn output_data_callback(
        buf: &mut [f32],
        info: &OutputCallbackInfo,
        output_channel: &Receiver<Vec<f32>>,
    ) {
        let values = match output_channel.recv() {
            Ok(values) => values,
            Err(e) => {
                log::warn!("Output Device: Sender closed channel: {}", e);
                return;
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
