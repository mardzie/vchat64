use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Neg,
    sync::{Arc, atomic},
};

use crate::{
    audio::traits::SampleFormatConversion,
    traits::num::{Num, NumAssign, NumPartialCmp},
};

#[derive(Debug)]
pub struct AudioProcessor<I, O>
where
    I: SampleFormatConversion<O> + Send + Sync + 'static,
    O: Debug + Display + Num + NumAssign + Neg + NumPartialCmp + Send + 'static,
    Vec<O>: Clone + FromIterator<O>,
{
    volume: Arc<atomic::AtomicU8>,

    input_sample_format: cpal::SampleFormat,

    _phantom_data_in: PhantomData<I>,
    _phantom_data_out: PhantomData<O>,
}

impl<I, O> AudioProcessor<I, O>
where
    I: SampleFormatConversion<O> + Send + Sync + 'static,
    O: Debug + Display + Num + NumAssign + Neg + NumPartialCmp + Send + 'static,
    Vec<O>: Clone + FromIterator<I>,
{
    pub fn new(volume: Arc<atomic::AtomicU8>, input_sample_format: cpal::SampleFormat) -> Self {
        Self {
            volume,

            input_sample_format,

            _phantom_data_in: PhantomData,
            _phantom_data_out: PhantomData,
        }
    }

    pub fn process_audio(&self, input_samples: Vec<I>) -> Vec<O> {
        let buf: Vec<O> = I::to_sample_buf(input_samples, Some(self.input_sample_format)).collect();

        log::trace!("Audio Processor: Processing sample {} bytes", buf.len() * 4);

        // Process audio
        // TODO

        buf
    }
}
