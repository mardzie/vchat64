use std::{
    marker::PhantomData,
    sync::{Arc, atomic},
};

use crate::traits::SampleFormatConversion;

#[derive(Debug)]
pub struct AudioProcessor<I, O> {
    volume: Arc<atomic::AtomicU8>,

    input_sample_format: cpal::SampleFormat,

    _phantom_data_in: PhantomData<I>,
    _phantom_data_out: PhantomData<O>,
}

impl<I, O> AudioProcessor<I, O> {
    pub fn new(volume: Arc<atomic::AtomicU8>, input_sample_format: cpal::SampleFormat) -> Self
    where
        I: SampleFormatConversion<f32> + Send + Sync + 'static,
        O: Send + 'static,
        Vec<O>: Clone + FromIterator<f32>,
    {
        Self {
            volume,

            input_sample_format,

            _phantom_data_in: PhantomData,
            _phantom_data_out: PhantomData,
        }
    }

    pub fn process_audio(&self, input_samples: Vec<I>) -> Vec<O>
    where
        I: SampleFormatConversion<f32>,
        Vec<O>: FromIterator<f32>,
    {
        let buf: Vec<O> = I::to_sample_buf(input_samples, Some(self.input_sample_format)).collect();

        log::trace!("Audio Processor: Processing sample {} bytes", buf.len() * 4);

        // Process audio
        // TODO

        buf
    }
}
