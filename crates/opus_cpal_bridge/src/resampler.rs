use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use cpal::{SAMPLE_RATE_48K, SupportedStreamConfig};

use crate::{
    resampler::resampler_inner::ResamplerInner,
    stream::stream_inner::{Input, Output},
};

mod resampler_inner;

#[derive(Debug)]
pub struct Resampler<D> {
    inner: ResamplerInner,

    _direction: PhantomData<D>,
}

impl Resampler<Input> {
    pub fn new(input_config: &SupportedStreamConfig) -> Self {
        Self {
            inner: ResamplerInner::new(
                input_config.sample_rate() as usize,
                SAMPLE_RATE_48K as usize,
            ),

            _direction: PhantomData,
        }
    }
}

impl Resampler<Output> {
    pub fn new(output_config: &SupportedStreamConfig) -> Self {
        Self {
            inner: ResamplerInner::new(
                SAMPLE_RATE_48K as usize,
                output_config.sample_rate() as usize,
            ),

            _direction: PhantomData,
        }
    }
}

impl<D> Deref for Resampler<D> {
    type Target = ResamplerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<D> DerefMut for Resampler<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
