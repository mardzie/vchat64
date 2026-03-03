use std::{
    iter::Peekable,
    sync::{
        Arc,
        atomic::{self, AtomicBool},
        mpsc::Sender,
    },
};

use cpal::{Host, InputCallbackInfo, OutputCallbackInfo, SampleFormat, default_host};

use crate::{
    audio::{audio_processor::AudioProcessor, input::InputStream, output::OutputStream},
    traits::SampleFormatCenter,
};

pub mod audio_processor;
pub mod config_filter;
pub mod error;
pub mod input;
pub mod output;
pub mod sample_format_center_impl;
pub mod sample_format_conversion_impl;

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
        output_channel: crossbeam::channel::Receiver<Vec<f32>>,

        init_volume: u8,
        init_cutoff: u8,

        exit: Arc<AtomicBool>,
    ) -> Self {
        let host = default_host();

        let (input_tx_to_audio_processor, audio_processor_rx) = std::sync::mpsc::channel();
        let mut input = InputStream::new(&host).expect("Failed to create new input object.");
        log::info!("Input Stream: Using config: {:?}", input.config());
        input
            .build_stream(
                move |buf, info| Self::input_data_callback(buf, info, &input_tx_to_audio_processor),
                move |e| log::error!("Input Stream Error: {}", e),
            )
            .expect("Failed to create new input stream.");

        let mut output = OutputStream::new(&host).expect("Failed to create new output object.");
        log::info!("Output Stream: Using config: {:?}", output.config());
        let output_sample_format = output.sample_format();
        output
            .build_stream(
                move |buf, info| {
                    Self::output_data_callback(
                        buf,
                        info,
                        &mut output_channel.try_iter().peekable(),
                        output_sample_format,
                    )
                },
                move |e| log::error!("Output Stream Error: {}", e),
            )
            .expect("Failed to create new output stream.");

        let volume = Arc::new(atomic::AtomicU8::new(init_volume));
        let cutoff = Arc::new(atomic::AtomicU8::new(init_cutoff));

        let volume_c = volume.clone();
        let cutoff_c = cutoff.clone();
        let input_sample_format = input.sample_format();
        let audio_processor = AudioProcessor::new(
            audio_processor_rx,
            input_channel,
            volume_c,
            cutoff_c,
            exit,
            input_sample_format,
        );

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
    fn input_data_callback<T>(buf: &[T], info: &InputCallbackInfo, input_channel: &Sender<Vec<T>>)
    where
        T: Copy,
    {
        match input_channel.send(buf.to_vec()) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Input Device: Receiver closed channel stopping input device: {}",
                    e
                )
            }
        };

        log::trace!(
            "Input Data Callback: Got called and read {} bytes",
            buf.len()
        );
    }

    #[inline]
    fn output_data_callback<T>(
        buf: &mut [T],
        info: &OutputCallbackInfo,
        output_channel: &mut Peekable<crossbeam::channel::TryIter<Vec<T>>>,
        sample_format: SampleFormat,
    ) where
        T: Copy + SampleFormatCenter,
    {
        let sample = match output_channel.next() {
            Some(sample) => sample,
            None => {
                log::warn!("Output Device: Sender closed channel");
                return;
            }
        };

        log::trace!("Output Data Callback: Got sample {} bytes", sample.len());
        
        let buf_len = buf.len();
        let mut buf_used = 0;

        Self::try_fill_buf(output_channel, buf, &mut buf_used, buf_len);

        // Fill remaining with silence.
        buf[buf_used..buf_len].fill(T::center_point(Some(sample_format)));
    }

    /// Tries to fill remaining `buf` space from `output_channel`.
    fn try_fill_buf<T>(
        output_channel: &mut Peekable<crossbeam::channel::TryIter<Vec<T>>>,
        buf: &mut [T],
        buf_used: &mut usize,
        buf_len: usize,
    ) where
        T: Copy,
    {
        while *buf_used < buf_len
            && let Some(sample) = output_channel.peek_mut()
        {
            let buf_space = buf_len - *buf_used;
            let sample_len = sample.len();

            if sample_len > buf_space {
                let extracted: Vec<T> = sample.drain(..buf_space).collect();
                buf[*buf_used..buf_len].copy_from_slice(&extracted);
                *buf_used = buf_len;
                log::trace!(
                    "Output Data Callback: Filled remaining space with {} of {} samples from new sample",
                    buf_space,
                    sample_len
                );
            } else {
                let sample = output_channel
                    .next()
                    .expect("Has to be `Some`. Outer loop checked for it.");
                let new_used = *buf_used + sample_len;
                buf[*buf_used..new_used].copy_from_slice(&sample);
                *buf_used = new_used;
                log::trace!(
                    "Output Data Callback: Filled space with {} samples of full extra sample, remaining space {}",
                    sample_len,
                    buf_len - *buf_used
                );
            };
        }
    }

    pub fn input_sample_format(&self) -> cpal::SampleFormat {
        self.input.sample_format()
    }

    pub fn output_sample_format(&self) -> cpal::SampleFormat {
        self.output.sample_format()
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

    pub fn stop(self) {
        self.audio_processor.stop();
    }
}

#[cfg(test)]
mod audio_test {
    use crate::audio::Audio;

    #[test]
    fn test_try_fill_remaining_single_recv_clean() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0, 5.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::try_fill_buf(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_try_fill_remaining_multi_recv_clean() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0]).unwrap();
        tx.send(vec![4.0, 5.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::try_fill_buf(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_try_fill_remaining_single_recv_remaining() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::try_fill_buf(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(rx.next().unwrap(), [6.0, 7.0]);
    }

    #[test]
    fn test_try_fill_remaining_multi_recv_remaining() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0]).unwrap();
        tx.send(vec![5.0, 6.0, 7.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::try_fill_buf(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(rx.next().unwrap(), [6.0, 7.0]);
    }

    fn get_setup<'a>() -> (
        [f32; 6],
        crossbeam::channel::Sender<Vec<f32>>,
        crossbeam::channel::Receiver<Vec<f32>>,
    ) {
        let buf = [0.0f32; 6];
        let (tx, rx) = crossbeam::channel::unbounded::<Vec<f32>>();
        (buf, tx, rx)
    }
}
