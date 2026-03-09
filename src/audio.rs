use std::{
    iter::Peekable,
    sync::{Arc, atomic},
};

use cpal::{Host, InputCallbackInfo, OutputCallbackInfo, SampleFormat, SizedSample, default_host};
use crossbeam::channel::{Receiver, Sender, TryIter};

use crate::{
    audio::{audio_processor::AudioProcessor, input::InputStream, output::OutputStream},
    traits::{SampleFormatCenter, SampleFormatConversion},
    vchat::AUDIO_CHANNELS_BUF_SIZE,
};

pub mod audio_processor;
pub mod config_filter;
pub mod error;
pub mod input;
pub mod output;
pub mod sample_format_center_impl;
pub mod sample_format_conversion_impl;

pub struct Audio<I, O> {
    host: Host,
    input: InputStream,
    output: OutputStream,
    audio_processor: AudioProcessor<I, O>,

    volume: Arc<atomic::AtomicU8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum InputMessage<T> {
    Samples(Vec<T>),
    Exit,
}

impl<I, O> Audio<I, O> {
    /// Creates a new [`Audio`] instance.
    ///
    /// `input_channel` is the channel where the microphone data gets sent after processing.
    ///
    /// `output_channel` is the channel that connects to the speaker.
    pub fn new(
        input_channel: Sender<InputMessage<I>>,
        output_channel: Receiver<Vec<O>>,

        init_volume: u8,
    ) -> (Self, Sender<InputMessage<I>>)
    where
        I: SizedSample + SampleFormatCenter + SampleFormatConversion<f32> + Send + Sync + 'static,
        O: SizedSample + SampleFormatCenter + Copy + Send + 'static,
        Vec<O>: FromIterator<f32>,
    {
        let host = default_host();

        let (input_tx_to_audio_processor, audio_processor_rx) =
            crossbeam::channel::bounded(AUDIO_CHANNELS_BUF_SIZE);
        let mut input = InputStream::new(&host).expect("Failed to create new input object.");
        log::info!("Input Stream: Using config: {:?}", input.config());
        let input_tx_to_audio_processor_c = input_tx_to_audio_processor.clone();
        input
            .build_stream(
                move |buf, info| {
                    Self::input_data_callback(buf, info, &input_tx_to_audio_processor_c)
                },
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

        let volume_c = volume.clone();
        let input_sample_format = input.sample_format();
        let audio_processor = AudioProcessor::<I, O>::new(volume_c, input_sample_format);

        (
            Self {
                host,
                input,
                output,
                audio_processor,

                volume,
            },
            input_tx_to_audio_processor,
        )
    }

    #[inline]
    fn input_data_callback(
        buf: &[I],
        info: &InputCallbackInfo,
        input_channel: &Sender<InputMessage<I>>,
    ) where
        I: SizedSample + Copy,
    {
        match input_channel.send(InputMessage::Samples(buf.to_vec())) {
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
    fn output_data_callback(
        buf: &mut [O],
        info: &OutputCallbackInfo,
        output_channel: &mut Peekable<TryIter<Vec<O>>>,
        sample_format: SampleFormat,
    ) where
        O: Copy + SampleFormatCenter,
    {
        let buf_len = buf.len();
        let mut buf_used = 0;

        Self::try_fill_buf(output_channel, buf, &mut buf_used, buf_len);

        // Fill remaining with silence.
        buf[buf_used..buf_len].fill(O::center_point(Some(sample_format)));
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
            && let Some(samples) = output_channel.peek_mut()
        {
            let buf_space = buf_len - *buf_used;
            let sample_len = samples.len();

            if sample_len > buf_space {
                let extracted: Vec<T> = samples.drain(..buf_space).collect();
                buf[*buf_used..buf_len].copy_from_slice(&extracted);
                *buf_used = buf_len;
                log::trace!(
                    "Output Data Callback: Filled remaining space with {} of {} samples from new sample",
                    buf_space,
                    sample_len
                );
            } else {
                let samples = output_channel
                    .next()
                    .expect("The peeked value was Samples but now it isn't anymore.");

                let new_used = *buf_used + sample_len;
                buf[*buf_used..new_used].copy_from_slice(&samples);
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

        Audio::<f32, f32>::try_fill_buf::<f32>(&mut rx, &mut buf, &mut buf_used, buf_len);

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

        Audio::<f32, f32>::try_fill_buf::<f32>(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_try_fill_remaining_single_recv_remaining() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::<f32, f32>::try_fill_buf::<f32>(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(rx.next().unwrap(), vec![6.0, 7.0]);
    }

    #[test]
    fn test_try_fill_remaining_multi_recv_remaining() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0]).unwrap();
        tx.send(vec![5.0, 6.0, 7.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::<f32, f32>::try_fill_buf::<f32>(&mut rx, &mut buf, &mut buf_used, buf_len);

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(rx.next().unwrap(), vec![6.0, 7.0]);
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
