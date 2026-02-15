use crate::traits::SampleFormatConversion;

impl SampleFormatConversion<f32> for f32 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self
    }

    fn convert_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for f64 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for u8 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        (self as f32 / Self::MAX as f32) * 2.0 - 1.0
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for u16 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        (self as f32 / Self::MAX as f32) * 2.0 - 1.0
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for u32 {
    fn convert_sample(self, sample_format: Option<cpal::SampleFormat>) -> f32 {
        match sample_format.unwrap_or(cpal::SampleFormat::U32) {
            cpal::SampleFormat::U24 => (self as f32 / ((1 << 24) - 1) as f32) * 2.0 - 1.0,
            cpal::SampleFormat::U32 => (self as f32 / Self::MAX as f32) * 2.0 - 1.0,
            format => {
                panic!(
                    "Can not call `convert_sample` on `u32` with `SampleFormat::{}`! Choose either `U24`, `U32` or `None`. `U32` and `None` produce the same output.",
                    format,
                );
            }
        }
    }

    fn convert_buf(
        buf: Vec<f32>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter()
            .map(move |sample| sample.convert_sample(sample_format))
    }
}

impl SampleFormatConversion<f32> for u64 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        (self as f32 / Self::MAX as f32) * 2.0 - 1.0
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for i8 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for i16 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}

impl SampleFormatConversion<f32> for i32 {
    fn convert_sample(self, sample_format: Option<cpal::SampleFormat>) -> f32 {
        match sample_format.unwrap_or(cpal::SampleFormat::I32) {
            cpal::SampleFormat::I24 => self as f32 / ((1 << 23) - 1) as f32,
            cpal::SampleFormat::I32 => self as f32 / Self::MAX as f32,
            format => {
                panic!(
                    "Can not call `convert_sample` on `i32` with `SampleFormat::{}`! Choose either `I24`, `I32` or `None`. `I32` and `None` produce the same output.",
                    format
                )
            }
        }
    }

    fn convert_buf(
        buf: Vec<f32>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter()
            .map(move |sample| sample.convert_sample(sample_format))
    }
}

impl SampleFormatConversion<f32> for i64 {
    fn convert_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32
    }

    fn convert_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> f32> {
        buf.into_iter().map(|sample| sample.convert_sample(None))
    }
}
