use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::Neg,
    sync::{Arc, atomic},
};

use crate::traits::{
    SampleFormatConversion,
    num::{Num, NumAssign, NumCmp, NumSh, NumShAssign},
};

#[derive(Debug)]
pub struct AudioProcessor<I, O, F>
where
    I: SampleFormatConversion<F> + Send + Sync + 'static,
    O: Debug + Display + Num + NumAssign + Neg + NumSh + NumShAssign + NumCmp + Send + 'static,
    Vec<O>: Clone + FromIterator<F>,
{
    volume: Arc<atomic::AtomicU8>,

    input_sample_format: cpal::SampleFormat,

    _phantom_data_in: PhantomData<I>,
    _phantom_data_out: PhantomData<O>,
    _phantom_data_format: PhantomData<F>,
}

impl<I, O, F> AudioProcessor<I, O, F>
where
    I: SampleFormatConversion<F> + Send + Sync + 'static,
    O: Debug + Display + Num + NumAssign + Neg + NumSh + NumShAssign + NumCmp + Send + 'static,
    Vec<O>: Clone + FromIterator<F>,
{
    pub fn new(volume: Arc<atomic::AtomicU8>, input_sample_format: cpal::SampleFormat) -> Self {
        Self {
            volume,

            input_sample_format,

            _phantom_data_in: PhantomData,
            _phantom_data_out: PhantomData,
            _phantom_data_format: PhantomData,
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
