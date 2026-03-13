use std::{
    fmt::{Debug, Display},
    ops::Neg,
};

use cpal::SizedSample;

use crate::traits::num::{Num, NumAssign, NumPartialCmp};

pub mod audio_input_impl;
pub mod audio_output_impl;
pub mod sample_format_center_impl;
pub mod sample_format_conversion_impl;

pub trait SampleFormatConversion<T> {
    /// Converts a sample `self` into `T`.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `SampleFormat` else it will be interpreted as `U32` or `I32`.
    fn to_sample(self, sample_format: Option<cpal::SampleFormat>) -> T;

    /// Converts a sample list `buf` into an iterator with `T` contents.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `sample_format` else it will be interpreted as `U32` or `I32`.
    fn to_sample_buf(
        buf: Vec<Self>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> T>
    where
        Self: Sized;

    /// Converts a sample `T` into `Self`.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `SampleFormat` else it will be interpreted as `U32` or `I32`.
    fn from_sample(sample: T, sample_format: Option<cpal::SampleFormat>) -> Self;

    /// Converts a list `buf` into an iterator with `Self` contents.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `sample_format` else it will be interpreted as `U32` or `I32`.
    fn from_sample_buf(
        buf: Vec<T>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<T>, impl FnMut(T) -> Self>
    where
        Self: Sized;
}

pub trait SampleFormatCenter {
    /// Returns the center point of this sample type.
    fn center_point(sample_format: Option<cpal::SampleFormat>) -> Self;
}

pub trait AudioInputTrait<O>:
    SizedSample + SampleFormatCenter + SampleFormatConversion<O> + Copy + Send + Sync + 'static
{
}

pub trait AudioOutputTrait:
    Debug
    + Display
    + Num
    + NumAssign
    + Neg
    + NumPartialCmp
    + SizedSample
    + SampleFormatCenter
    + Copy
    + Send
    + 'static
{
}
