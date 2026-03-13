use cpal::SizedSample;

use crate::audio::traits::{AudioInputTrait, SampleFormatCenter, SampleFormatConversion};

impl<O> AudioInputTrait<O> for O where
    O: SampleFormatConversion<O> + SampleFormatCenter + SizedSample + Send + Sync + 'static
{
}
