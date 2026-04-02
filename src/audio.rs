use std::{
    fmt::Debug,
    iter::Peekable,
    sync::{Arc, atomic},
};

use cpal::{Host, InputCallbackInfo, OutputCallbackInfo, SampleFormat, SizedSample, default_host};
use crossbeam::channel::{Receiver, Sender, TryIter};
use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Producer, Split},
    wrap::caching::Caching,
};

use crate::audio::{
    audio_processor::AudioProcessor,
    input::InputStream,
    output::OutputStream,
    traits::{NormalizeSample, SampleOrigin, copy_from_iter_impl::CopyFromIterator},
};

pub mod audio_processor;
pub mod config_filter;
pub mod error;
pub mod input;
pub mod output;
pub mod traits;

pub const AUDIO_RING_BUF_CAPACITY: usize = 1024 * 64;

pub struct Audio {
    host: Host,
    input: InputStream,
    output: OutputStream,
    audio_processor: Arc<AudioProcessor>,

    volume: Arc<atomic::AtomicU8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum InputMessage {
    Samples,
    Exit,
}

impl Audio {
    /// Creates a new [`Audio`] instance.
    ///
    /// `input_channel` is the channel where the microphone data gets sent after processing.
    ///
    /// `output_channel` is the channel that connects to the speaker.
    pub fn new(
        input_notify: Sender<InputMessage>,
        output_channel: Receiver<Vec<f32>>,
        init_volume: u8,
    ) -> (Self, Caching<Arc<SharedRb<Heap<f32>>>, false, true>) {
        let host = default_host();

        let ring_buf = ringbuf::HeapRb::<f32>::new(AUDIO_RING_BUF_CAPACITY);
        let (mut producer, consumer) = ring_buf.split();
        let mut input = InputStream::new(&host).expect("Failed to create new input object.");
        let input_sample_format = input.sample_format();
        let input_sample_format_c = input_sample_format.clone();
        log::info!("Input Stream: Using config: {:?}", input.config());
        input
            .build_stream(
                move |buf, info| {
                    Self::input_data_callback(
                        buf,
                        info,
                        &input_notify,
                        &mut producer,
                        &input_sample_format_c,
                    )
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
                        &output_sample_format,
                    )
                },
                move |e| log::error!("Output Stream Error: {}", e),
            )
            .expect("Failed to create new output stream.");

        let volume = Arc::new(atomic::AtomicU8::new(init_volume));

        let volume_c = volume.clone();
        let audio_processor = Arc::new(AudioProcessor::new(volume_c, input_sample_format));

        (
            Self {
                host,
                input,
                output,
                audio_processor,

                volume,
            },
            consumer,
        )
    }

    #[inline(always)]
    fn input_data_callback<T>(
        buf: &[T],
        info: &InputCallbackInfo,
        input_notify: &Sender<InputMessage>,
        producer: &mut Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
        sample_format: &SampleFormat,
    ) where
        T: Copy + SizedSample + NormalizeSample<f32>,
    {
        producer.push_iter(
            buf.iter()
                .map(|sample| sample.normalize(Some(&sample_format))),
        );
        let _ = input_notify.try_send(InputMessage::Samples);
    }

    #[inline(always)]
    fn output_data_callback<T>(
        buf: &mut [T],
        info: &OutputCallbackInfo,
        output_channel: &mut Peekable<TryIter<Vec<f32>>>,
        sample_format: &SampleFormat,
    ) where
        T: Clone + Copy + NormalizeSample<f32> + SampleOrigin,
    {
        let buf_len = buf.len();
        let mut buf_used = 0;

        Self::try_fill_buf(output_channel, buf, &mut buf_used, buf_len, sample_format);

        // Fill remaining with silence.
        buf[buf_used..buf_len].fill(T::origin(Some(sample_format)));
    }

    /// Tries to fill remaining `buf` space from `output_channel`.
    fn try_fill_buf<T>(
        output_channel: &mut Peekable<crossbeam::channel::TryIter<Vec<f32>>>,
        buf: &mut [T],
        buf_used: &mut usize,
        buf_len: usize,
        sample_format: &SampleFormat,
    ) where
        T: Copy + NormalizeSample<f32>,
    {
        while *buf_used < buf_len
            && let Some(samples) = output_channel.peek_mut()
        {
            let buf_space = buf_len - *buf_used;
            let sample_len = samples.len();

            if sample_len > buf_space {
                let extracted = samples
                    .drain(..buf_space)
                    .map(|sample| T::denormalize(sample, Some(sample_format)));
                buf[*buf_used..buf_len].copy_from_iter(extracted);
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
                buf[*buf_used..new_used]
                    .copy_from_iter(T::denormalize_buf(samples, Some(sample_format)));
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

    pub fn audio_processor(&self) -> Arc<AudioProcessor> {
        self.audio_processor.clone()
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

        Audio::try_fill_buf::<f32>(
            &mut rx,
            &mut buf,
            &mut buf_used,
            buf_len,
            &cpal::SampleFormat::F32,
        );

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

        Audio::try_fill_buf::<f32>(
            &mut rx,
            &mut buf,
            &mut buf_used,
            buf_len,
            &cpal::SampleFormat::F32,
        );

        assert_eq!(buf, [0.0, 0.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_try_fill_remaining_single_recv_remaining() {
        let (mut buf, tx, rx) = get_setup();
        let mut rx = rx.try_iter().peekable();

        tx.send(vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0]).unwrap();

        let mut buf_used = 2;
        let buf_len = buf.len();

        Audio::try_fill_buf::<f32>(
            &mut rx,
            &mut buf,
            &mut buf_used,
            buf_len,
            &cpal::SampleFormat::F32,
        );

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

        Audio::try_fill_buf::<f32>(
            &mut rx,
            &mut buf,
            &mut buf_used,
            buf_len,
            &cpal::SampleFormat::F32,
        );

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
