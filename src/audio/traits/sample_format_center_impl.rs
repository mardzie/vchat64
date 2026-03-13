use crate::audio::traits::SampleFormatCenter;

impl SampleFormatCenter for f32 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        0.0
    }
}

impl SampleFormatCenter for f64 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        0.0
    }
}

impl SampleFormatCenter for u8 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        1 << 7
    }
}

impl SampleFormatCenter for u16 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        1 << 15
    }
}

impl SampleFormatCenter for u32 {
    fn center_point(sample_format: Option<cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(cpal::SampleFormat::U32) {
            cpal::SampleFormat::U24 => 1 << 23,
            cpal::SampleFormat::U32 => 1 << 31,
            format => panic!(
                "Can not call `center_point` on `u32` with `SampleFormat::{}`! Choose either `U24`, `U32` or `None`. `U32` and `None` produce the same output.",
                format
            ),
        }
    }
}

impl SampleFormatCenter for u64 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        1 << 63
    }
}

impl SampleFormatCenter for i8 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        0
    }
}

impl SampleFormatCenter for i16 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        0
    }
}

impl SampleFormatCenter for i32 {
    fn center_point(sample_format: Option<cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(cpal::SampleFormat::I32) {
            cpal::SampleFormat::I24 | cpal::SampleFormat::I32 => 0,
            format => panic!(
                "Can not call `center_point` on `i32` with `SampleFormat::{}`! Choose either `I24`, `I32` or `None`. `I32` and `None` produce the same output.",
                format
            ),
        }
    }
}

impl SampleFormatCenter for i64 {
    fn center_point(_: Option<cpal::SampleFormat>) -> Self {
        0
    }
}
