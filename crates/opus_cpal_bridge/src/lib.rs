use std::{fmt::Debug, sync::Arc};

use cpal::{Host, SAMPLE_RATE_48K};
use ringbuf::{
    SharedRb,
    storage::Heap,
    traits::{Consumer, Observer, Split},
    wrap::caching::Caching,
};

pub use opus::Bitrate;
use rubato::{Fft, Resampler as _, audioadapter_buffers::direct::SequentialSlice};

use crate::{
    error::AudioBridgeNewError,
    macros::build_stream::{build_input_stream, build_output_stream},
    stream::{DeviceType, Stream},
    stream_trait::StreamControls,
};

mod macros;
mod stream;

pub mod error;
pub mod stream_trait;

const RINGBUF_SIZE: usize = 4 * 1024;
const FRAME: usize = (SAMPLE_RATE_48K as f32 * 0.020) as usize; // The sample rate of 48K is used and opus takes frames in 20 ms.

pub const OPUS_DEFAULT_BITRATE: i32 = 24_000;

pub struct AudioBridge {
    host: Host,

    input_stream: Stream,
    input_consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
    /// The maximum frames before trimming. If frames_len > max_backlog it will get trimmed.
    max_backlog: usize,
    input_scratch_buf: Vec<f32>,
    input_resampled_buf: Vec<f32>,
    input_resampler: Resampler,
    encoder: opus::Encoder,

    output_stream: Stream,
    output_producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
    output_scratch_buf: Vec<f32>,
    output_resampled_buf: Vec<f32>,
    output_resampler: Resampler,
    decoder: opus::Decoder,
}

impl AudioBridge {
    pub fn new() -> Result<Self, AudioBridgeNewError> {
        let host = cpal::default_host();
        let mut input_stream = Stream::new(DeviceType::Input, &host)?;
        let mut output_stream = Stream::new(DeviceType::Output, &host)?;

        let input_ringbuf: SharedRb<Heap<f32>> = ringbuf::SharedRb::new(RINGBUF_SIZE);
        let (mut input_producer, input_consumer) = input_ringbuf.split();
        let output_ringbuf: SharedRb<Heap<f32>> = ringbuf::SharedRb::new(RINGBUF_SIZE);
        let (output_producer, mut output_consumer) = output_ringbuf.split();

        let input_config = input_stream.config();
        let output_config = output_stream.config();
        build_input_stream!(input_stream, audio_callback::input_audio_callback, (&mut input_producer, input_config.channels() as usize), {
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

        let input_resampler = Resampler::new(
            input_config.sample_rate() as usize,
            SAMPLE_RATE_48K as usize,
        );
        let output_resampler = Resampler::new(
            SAMPLE_RATE_48K as usize,
            output_config.sample_rate() as usize,
        );

        let mut encoder = opus::Encoder::new(
            SAMPLE_RATE_48K,
            opus::Channels::Mono,
            opus::Application::Voip,
        )?;
        encoder.set_bitrate(Bitrate::Bits(OPUS_DEFAULT_BITRATE))?;
        encoder.set_vbr(true)?;
        encoder.set_inband_fec(true)?;
        encoder.set_packet_loss_perc(10)?;
        let decoder = opus::Decoder::new(SAMPLE_RATE_48K, opus::Channels::Mono)?;

        input_stream.play().expect("Failed to start input stream");
        output_stream.play().expect("Failed to start output stream");

        Ok(Self {
            host,

            input_stream,
            input_consumer,
            max_backlog: 4 * FRAME,
            input_scratch_buf: vec![0f32; input_resampler.max_input()],
            input_resampled_buf: vec![0f32; input_resampler.max_output()],
            input_resampler,
            encoder,

            output_stream,
            output_producer,
            output_scratch_buf: vec![0f32; output_resampler.max_input()],
            output_resampled_buf: vec![0f32; output_resampler.max_output()],
            output_resampler,
            decoder,
        })
    }

    /// Pops an opus frame.
    ///
    /// When None is returned retry after some time (~5 ms).
    pub fn pop(&mut self, packet: &mut [u8]) -> Option<usize> {
        self.trim_backlog();

        let input_frames_needed = self.input_resampler.input_frames_next();
        debug_assert!(self.max_backlog >= input_frames_needed);
        if self.input_consumer.occupied_len() < input_frames_needed {
            return None;
        }

        let popped_frames = self.input_consumer.pop_slice(&mut self.input_scratch_buf);
        debug_assert_eq!(popped_frames, input_frames_needed);

        self.input_resampler.resample(
            &self.input_scratch_buf[..input_frames_needed],
            &mut self.input_resampled_buf,
        );

        let len = self
            .encoder
            .encode_float(&self.input_resampled_buf, packet)
            .expect("Failed to encode to opus frame");

        Some(len)
    }

    pub fn push(&mut self) {}

    pub fn set_bitrate(&mut self, bitrate: Bitrate) -> Result<(), opus::Error> {
        self.encoder.set_bitrate(bitrate)
    }

    pub fn get_bitrate(&mut self) -> Result<Bitrate, opus::Error> {
        self.encoder.get_bitrate()
    }

    pub fn set_packet_loss_perc(&mut self, packet_loss_percent: i32) -> Result<(), opus::Error> {
        self.encoder.set_packet_loss_perc(packet_loss_percent)
    }

    fn trim_backlog(&mut self) {
        let available_frames = self.input_consumer.occupied_len();
        if available_frames > self.max_backlog {
            self.input_consumer
                .skip(available_frames - self.max_backlog);
        }
    }
}

impl Debug for AudioBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioBridge").finish()
    }
}

#[derive(Debug)]
enum Resampler {
    Native,
    Convert(Fft<f32>),
}

impl Resampler {
    pub fn new(sample_rate_input: usize, sample_rate_output: usize) -> Self {
        if sample_rate_input == SAMPLE_RATE_48K as usize {
            Self::Native
        } else {
            Self::Convert(
                rubato::Fft::<f32>::new(
                    sample_rate_input,
                    sample_rate_output,
                    FRAME,
                    1,
                    rubato::FixedSync::Output,
                )
                .expect("Failed to create resampler"),
            )
        }
    }

    pub fn max_input(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => resampler.input_frames_max(),
        }
    }

    pub fn max_output(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => resampler.output_frames_max(),
        }
    }

    /// Get the number of frames per channel needed.
    pub fn input_frames_next(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => resampler.input_frames_next(),
        }
    }

    pub fn resample(&mut self, in_buf: &[f32], out_buf: &mut [f32]) -> () {
        match self {
            Resampler::Native => {
                debug_assert_eq!(in_buf.len(), FRAME);
                out_buf[..FRAME].copy_from_slice(&in_buf[..FRAME])
            }
            Resampler::Convert(resampler) => {
                let input_adapter = SequentialSlice::new(in_buf, 1, in_buf.len())
                    .expect("input adapter size mismatch");
                let mut output_adapter = SequentialSlice::new_mut(out_buf, 1, FRAME)
                    .expect("output adapter size mismatch");

                let (_consumed, produced) = resampler
                    .process_into_buffer(&input_adapter, &mut output_adapter, None)
                    .expect("Failed to process samples into buffer");
                debug_assert_eq!(produced, FRAME);
            }
        };
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
