use rubato::{Fft, Resampler as _, audioadapter_buffers::direct::SequentialSlice};

use crate::{FRAME_SIZE, SAMPLE_RATE_48K, opus_codec::frame_size::FrameSize, stream::DeviceType};

/// `Resampler`
///
/// The `Resampler` is stateful.
/// One `Resampler` per stream.
#[derive(Debug)]
pub struct Resampler {
    inner: ResamplerInner,
    frame_size: FrameSize,
}

impl Resampler {
    #[must_use]
    pub fn new(device_type: DeviceType, config: cpal::SupportedStreamConfig) -> Self {
        Self::from_frame_size(device_type, config, FRAME_SIZE)
    }

    #[must_use]
    pub fn from_frame_size(
        device_type: DeviceType,
        config: cpal::SupportedStreamConfig,
        frame_size: FrameSize,
    ) -> Self {
        let inner = ResamplerInner::new(
            device_type,
            config,
            frame_size.samples(SAMPLE_RATE_48K) as u32,
        );
        Self { inner, frame_size }
    }

    /// The maximum input buffer size.
    #[must_use]
    #[inline]
    pub fn max_input(&self) -> usize {
        self.inner.max_input(self.frame_at_48k())
    }

    /// The maximum output buffer size.
    #[must_use]
    #[inline]
    pub fn max_output(&self) -> usize {
        self.inner.max_output(self.frame_at_48k())
    }

    /// Get the number of input frames per channel for the next call to `resample()`.
    #[must_use]
    #[inline]
    pub fn input_frames_next(&self) -> usize {
        self.inner.input_frames_next(self.frame_at_48k())
    }

    /// Get the number of output frames per channel for the next call to `resample()`
    #[must_use]
    #[inline]
    pub fn output_frames_next(&self) -> usize {
        self.inner.output_frames_next(self.frame_at_48k())
    }

    /// One side is always a fixed length.
    ///
    /// `DeviceType::Input` -> Output is a fixed size, Input dynamic.
    ///
    /// `DeviceType::Output` -> Input is a fixed size, Output dynamic.
    ///
    /// The dynamic side has to be queried with `input_frames_next()` or `output_frames_next()`.
    /// The slices may be greater than the requested size but only the requested frames will be processed.
    ///
    /// Returns the number of frames consumed and produced as a tuple `(consumed, produced)`.
    pub fn resample(
        &mut self,
        input_buf: &[f32],
        output_buf: &mut [f32],
    ) -> Result<(usize, usize), rubato::ResampleError> {
        self.inner
            .resample(input_buf, output_buf, self.frame_at_48k())
    }

    #[must_use]
    fn frame_at_48k(&self) -> usize {
        self.frame_size.samples(SAMPLE_RATE_48K)
    }
}

#[derive(Debug)]
enum ResamplerInner {
    Native,
    Convert(Fft<f32>),
}

impl ResamplerInner {
    #[must_use]
    pub fn new(
        device_type: DeviceType,
        config: cpal::SupportedStreamConfig,
        chunk_size: u32,
    ) -> Self {
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

        Self::from_sample_rates(
            device_type,
            sample_rate_input,
            sample_rate_output,
            chunk_size,
        )
    }

    #[must_use]
    pub fn from_sample_rates(
        device_type: DeviceType,
        sample_rate_input: usize,
        sample_rate_output: usize,
        chunk_size: u32,
    ) -> Self {
        if sample_rate_input == sample_rate_output {
            Self::Native
        } else {
            const _: () = assert!(
                usize::BITS >= u32::BITS,
                "only 32 bit and up targets are supported"
            );
            Self::Convert(
                rubato::Fft::<f32>::new(
                    sample_rate_input,
                    sample_rate_output,
                    chunk_size as usize,
                    1,
                    device_type.direction(),
                )
                .expect("Failed to create resampler"),
            )
        }
    }

    /// The maximum input buffer size.
    #[must_use]
    pub fn max_input(&self, frame: usize) -> usize {
        match self {
            ResamplerInner::Native => frame,
            ResamplerInner::Convert(resampler) => resampler.input_frames_max(),
        }
    }

    /// The maximum output buffer size.
    #[must_use]
    pub fn max_output(&self, frame: usize) -> usize {
        match self {
            ResamplerInner::Native => frame,
            ResamplerInner::Convert(resampler) => resampler.output_frames_max(),
        }
    }

    /// Get the number of input frames per channel for the next call to `resample()`.
    #[must_use]
    pub fn input_frames_next(&self, frame: usize) -> usize {
        match self {
            ResamplerInner::Native => frame,
            ResamplerInner::Convert(resampler) => Self::input_frames_next_inner(resampler),
        }
    }

    #[must_use]
    fn input_frames_next_inner(resampler: &Fft<f32>) -> usize {
        resampler.input_frames_next()
    }

    /// Get the number of output frames per channel for the next call to `resample()`
    #[must_use]
    pub fn output_frames_next(&self, frame: usize) -> usize {
        match self {
            ResamplerInner::Native => frame,
            ResamplerInner::Convert(resampler) => Self::output_frames_next_inner(resampler),
        }
    }

    #[must_use]
    fn output_frames_next_inner(resampler: &Fft<f32>) -> usize {
        resampler.output_frames_next()
    }

    pub fn resample(
        &mut self,
        input_buf: &[f32],
        output_buf: &mut [f32],
        frame: usize,
    ) -> Result<(usize, usize), rubato::ResampleError> {
        match self {
            ResamplerInner::Native => {
                debug_assert!(
                    input_buf.len() >= frame,
                    "Input buffer needs to be at least {frame} long",
                );
                debug_assert!(
                    output_buf.len() >= frame,
                    "Output buffer needs to be at least {frame} long",
                );

                output_buf[..frame].copy_from_slice(&input_buf[..frame]);
                Ok((frame, frame))
            }
            ResamplerInner::Convert(resampler) => {
                debug_assert!(
                    input_buf.len() >= Self::input_frames_next_inner(&resampler),
                    "Input buffer needs to be at least {} long",
                    Self::input_frames_next_inner(&resampler)
                );
                debug_assert!(
                    output_buf.len() >= Self::output_frames_next_inner(&resampler),
                    "Output buffer needs to be at least {} long",
                    Self::output_frames_next_inner(&resampler)
                );

                let input_adapter = SequentialSlice::new(input_buf, 1, input_buf.len())
                    .expect("input adapter size mismatch");
                let mut output_adapter = SequentialSlice::new_mut(output_buf, 1, frame)
                    .expect("output adapter size mismatch");

                let (consumed, produced) =
                    resampler.process_into_buffer(&input_adapter, &mut output_adapter, None)?;
                debug_assert_eq!(produced, frame);

                Ok((consumed, produced))
            }
        }
    }
}

trait ResampleDirection {
    fn direction(self) -> rubato::FixedSync;
}
impl ResampleDirection for DeviceType {
    fn direction(self) -> rubato::FixedSync {
        match self {
            Self::Input => rubato::FixedSync::Output,
            Self::Output => rubato::FixedSync::Input,
        }
    }
}
