use crate::audio::traits::SampleOrigin;

impl SampleOrigin for f32 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        0.0
    }
}

impl SampleOrigin for f64 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        0.0
    }
}

impl SampleOrigin for u8 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        1 << 7
    }
}

impl SampleOrigin for u16 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        1 << 15
    }
}

impl SampleOrigin for u32 {
    #[inline(always)]
    fn origin(sample_format: Option<&cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(&cpal::SampleFormat::U32) {
            cpal::SampleFormat::U24 => 1 << 23,
            cpal::SampleFormat::U32 => 1 << 31,
            format => panic!(
                "Can not call `center_point` on `u32` with `SampleFormat::{}`! Choose either `U24`, `U32` or `None`. `U32` and `None` produce the same output.",
                format
            ),
        }
    }
}

impl SampleOrigin for u64 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        1 << 63
    }
}

impl SampleOrigin for i8 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        0
    }
}

impl SampleOrigin for i16 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        0
    }
}

impl SampleOrigin for i32 {
    #[inline(always)]
    fn origin(sample_format: Option<&cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(&cpal::SampleFormat::I32) {
            cpal::SampleFormat::I24 | cpal::SampleFormat::I32 => 0,
            format => panic!(
                "Can not call `center_point` on `i32` with `SampleFormat::{}`! Choose either `I24`, `I32` or `None`. `I32` and `None` produce the same output.",
                format
            ),
        }
    }
}

impl SampleOrigin for i64 {
    #[inline(always)]
    fn origin(_: Option<&cpal::SampleFormat>) -> Self {
        0
    }
}
