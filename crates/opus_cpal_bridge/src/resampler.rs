use rubato::{Fft, Resampler as _, audioadapter_buffers::direct::SequentialSlice};

use crate::{FRAME, SAMPLE_RATE_48K, stream::DeviceType};

/// `Resampler`
///
/// The `Resampler` is stateful.
/// One `Resampler` per stream.
#[derive(Debug)]
pub enum Resampler {
    Native,
    Convert(Fft<f32>),
}

impl Resampler {
    #[must_use]
    pub fn new(device_type: DeviceType, config: cpal::SupportedStreamConfig) -> Self {
        let sample_rate_input: usize;
        let sample_rate_output: usize;
        match device_type {
            DeviceType::Input => {
                sample_rate_input = config.sample_rate() as usize;
                sample_rate_output = SAMPLE_RATE_48K as usize;
            }
            DeviceType::Output => {
                sample_rate_input = SAMPLE_RATE_48K as usize;
                sample_rate_output = config.sample_rate() as usize;
            }
        }

        Self::from_sample_rates(sample_rate_input, sample_rate_output)
    }

    #[must_use]
    pub fn from_sample_rates(sample_rate_input: usize, sample_rate_output: usize) -> Self {
        if sample_rate_input == sample_rate_output {
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

    /// The maximum input buffer size.
    #[must_use]
    pub fn max_input(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => resampler.input_frames_max(),
        }
    }

    /// The maximum output buffer size.
    #[must_use]
    pub fn max_output(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => resampler.output_frames_max(),
        }
    }

    /// Get the number of input frames per channel for the next call to `resample()`.
    #[must_use]
    #[inline]
    pub fn input_frames_next(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => Self::input_frames_next_inner(resampler),
        }
    }

    #[must_use]
    fn input_frames_next_inner(resampler: &Fft<f32>) -> usize {
        resampler.input_frames_next()
    }

    /// Get the number of output frames per channel for the next call to `resample()`
    #[must_use]
    #[inline]
    pub fn output_frames_next(&self) -> usize {
        match self {
            Resampler::Native => FRAME,
            Resampler::Convert(resampler) => Self::output_frames_next_inner(resampler),
        }
    }

    #[must_use]
    fn output_frames_next_inner(resampler: &Fft<f32>) -> usize {
        resampler.output_frames_next()
    }

    /// The input buffer size for the next call must be queried with `input_frames_next()` because it can change with each call to `resample()`.
    /// The output buffer size is fixed and can be queried with `output_frames_next()`.
    /// The slices may be greater than the requested size but only the requested frames will be processed.
    ///
    /// Returns the number of frames consumed and produced as a tuple `(consumed, produced)`.
    #[inline]
    pub fn resample(
        &mut self,
        input_buf: &[f32],
        output_buf: &mut [f32],
    ) -> Result<(usize, usize), rubato::ResampleError> {
        match self {
            Resampler::Native => {
                debug_assert!(
                    input_buf.len() >= FRAME,
                    "Input buffer needs to be at least {FRAME} long",
                );
                debug_assert!(
                    output_buf.len() >= FRAME,
                    "Output buffer needs to be at least {FRAME} long"
                );

                output_buf[..FRAME].copy_from_slice(&input_buf[..FRAME]);
                Ok((FRAME, FRAME))
            }
            Resampler::Convert(resampler) => {
                debug_assert!(
                    input_buf.len() >= Self::input_frames_next_inner(resampler),
                    "Input buffer needs to be at least {} long",
                    Self::input_frames_next_inner(resampler)
                );
                debug_assert!(
                    output_buf.len() >= Self::output_frames_next_inner(resampler),
                    "Output buffer needs to be at least {} long",
                    Self::output_frames_next_inner(resampler)
                );

                let input_adapter = SequentialSlice::new(input_buf, 1, input_buf.len())
                    .expect("input adapter size mismatch");
                let mut output_adapter = SequentialSlice::new_mut(output_buf, 1, FRAME)
                    .expect("output adapter size mismatch");

                let (consumed, produced) =
                    resampler.process_into_buffer(&input_adapter, &mut output_adapter, None)?;
                debug_assert_eq!(produced, FRAME);

                Ok((consumed, produced))
            }
        }
    }
}
