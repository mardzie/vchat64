use cpal::SAMPLE_RATE_48K;
use rubato::{Fft, Resampler as _, audioadapter_buffers::direct::SequentialSlice};

use crate::FRAME;

#[derive(Debug)]
pub enum ResamplerInner {
    Native,
    Convert(Fft<f32>),
}

impl ResamplerInner {
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
            ResamplerInner::Native => FRAME,
            ResamplerInner::Convert(resampler) => resampler.input_frames_max(),
        }
    }

    pub fn max_output(&self) -> usize {
        match self {
            ResamplerInner::Native => FRAME,
            ResamplerInner::Convert(resampler) => resampler.output_frames_max(),
        }
    }

    /// Get the number of input frames per channel for the next call to `resample()`.
    pub fn input_frames_next(&self) -> usize {
        match self {
            ResamplerInner::Native => FRAME,
            ResamplerInner::Convert(resampler) => resampler.input_frames_next(),
        }
    }

    /// Get the number of output frames per channel for the next call to `resample()`
    pub fn output_frames_next(&self) -> usize {
        match self {
            ResamplerInner::Native => FRAME,
            ResamplerInner::Convert(resampler) => resampler.output_frames_next(),
        }
    }

    pub fn resample(&mut self, input_buf: &[f32], output_buf: &mut [f32]) -> () {
        match self {
            ResamplerInner::Native => {
                debug_assert_eq!(input_buf.len(), FRAME);
                output_buf[..FRAME].copy_from_slice(&input_buf[..FRAME])
            }
            ResamplerInner::Convert(resampler) => {
                let input_adapter = SequentialSlice::new(input_buf, 1, input_buf.len())
                    .expect("input adapter size mismatch");
                let mut output_adapter = SequentialSlice::new_mut(output_buf, 1, FRAME)
                    .expect("output adapter size mismatch");

                let (_consumed, produced) = resampler
                    .process_into_buffer(&input_adapter, &mut output_adapter, None)
                    .expect("Failed to process samples into buffer");
                debug_assert_eq!(produced, FRAME);
            }
        };
    }
}
