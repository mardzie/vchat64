use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
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

        exit: Arc<AtomicBool>,
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
            AudioProcessor::new(audio_processor_rx, input_channel, volume_c, cutoff_c, exit);

        Self {
            input,
            output,
            audio_processor,
            host,

            volume,
            cutoff,
        }
    }

    #[inline]
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

    #[inline]
    fn output_data_callback(
        buf: &mut [f32],
        info: &OutputCallbackInfo,
        output_channel: &Receiver<Vec<f32>>,
    ) {
        let sample = match output_channel.recv() {
            Ok(sample) => sample,
            Err(e) => {
                log::warn!("Output Device: Sender closed channel: {}", e);
                return;
            }
        };

        let buf_len = buf.len();
        let mut buf_used = sample.len().min(buf_len);
        buf[..buf_used].copy_from_slice(&sample[..buf_used]);

        Self::try_fill_remaining(output_channel, buf, &mut buf_used, buf_len);
    }

    /// Tries to fill remaining `buf` space from `output_channel`.
    fn try_fill_remaining(
        output_channel: &Receiver<Vec<f32>>,
        buf: &mut [f32],
        buf_used: &mut usize,
        buf_len: usize,
    ) {
        let mut peekable = output_channel.iter().peekable();
        while *buf_used < buf_len
            && let Some(sample) = peekable.peek_mut()
        {
            let buf_space = buf_len - *buf_used;
            let sample_len = sample.len();
            if sample_len > buf_space {
                let extracted = sample.split_off(buf_space);
                buf[*buf_used..buf_len].copy_from_slice(&extracted);
            } else {
                let sample = peekable
                    .next()
                    .expect("Has to be `Some`. Outer loop checked for it.");
                let new_used = *buf_used + sample_len;
                buf[*buf_used..new_used].copy_from_slice(&sample);
                *buf_used = new_used
            };
        }
    }

    #[inline]
    pub fn play(&self) {
        self.play_input();
        self.play_output();
    }

    #[inline]
    pub fn play_input(&self) {
        self.input.play().expect("Failed to play input stream.");
    }

    #[inline]
    pub fn play_output(&self) {
        self.output.play().expect("Failed to play output stream.");
    }

    #[inline]
    pub fn pause(&self) {
        self.pause_input();
        self.pause_output();
    }

    #[inline]
    pub fn pause_input(&self) {
        self.input.pause().expect("Failed to pause input stream.");
    }

    #[inline]
    pub fn pause_output(&self) {
        self.output.pause().expect("Failed to pause output stream.");
    }

    pub fn stop(self) {
        self.audio_processor.stop();
    }
}
