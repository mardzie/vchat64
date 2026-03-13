use crate::audio::traits::SampleFormatConversion;

impl SampleFormatConversion<Self> for f32 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        sample
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for f64 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        sample as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for u8 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32 * 2.0 - 1.0
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        ((sample + 1.0) / 2.0 * Self::MAX as f32) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for u16 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32 * 2.0 - 1.0
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        ((sample + 1.0) / 2.0 * Self::MAX as f32) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for u32 {
    #[inline]
    fn to_sample(self, sample_format: Option<cpal::SampleFormat>) -> f32 {
        match sample_format.unwrap_or(cpal::SampleFormat::U32) {
            cpal::SampleFormat::U24 => self as f32 / ((1 << 24) - 1) as f32 * 2.0 - 1.0,
            cpal::SampleFormat::U32 => self as f32 / Self::MAX as f32 * 2.0 - 1.0,
            format => {
                panic!(
                    "Can not call `to_sample` on `u32` with `SampleFormat::{}`! Choose either `U24`, `U32` or `None`. `U32` and `None` produce the same output.",
                    format,
                );
            }
        }
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter()
            .map(move |sample| sample.to_sample(sample_format))
    }

    #[inline]
    fn from_sample(sample: f32, sample_format: Option<cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(cpal::SampleFormat::U32) {
            cpal::SampleFormat::U24 => ((sample + 1.0) / 2.0 * ((1 << 24) - 1) as f32) as Self,
            cpal::SampleFormat::U32 => ((sample + 1.0) / 2.0 * Self::MAX as f32) as Self,
            format => {
                panic!(
                    "Can not call `from_sample` on `u32` with `SampleFormat::{}`! Choose either `U24`, `U32` or `None`. `U32` and `None` produce the same output.",
                    format
                )
            }
        }
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(move |sample| Self::from_sample(sample, sample_format))
    }
}

impl SampleFormatConversion<f32> for u64 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        (self as f64 / Self::MAX as f64 * 2.0 - 1.0) as f32
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        ((sample as f64 + 1.0) / 2.0 * Self::MAX as f64) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for i8 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        (sample * Self::MAX as f32) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for i16 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        self as f32 / Self::MAX as f32
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        (sample * Self::MAX as f32) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for i32 {
    #[inline]
    fn to_sample(self, sample_format: Option<cpal::SampleFormat>) -> f32 {
        match sample_format.unwrap_or(cpal::SampleFormat::I32) {
            cpal::SampleFormat::I24 => self as f32 / ((1 << 23) - 1) as f32,
            cpal::SampleFormat::I32 => self as f32 / Self::MAX as f32,
            format => {
                panic!(
                    "Can not call `to_sample` on `i32` with `SampleFormat::{}`! Choose either `I24`, `I32` or `None`. `I32` and `None` produce the same output.",
                    format
                )
            }
        }
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter()
            .map(move |sample| sample.to_sample(sample_format))
    }

    #[inline]
    fn from_sample(sample: f32, sample_format: Option<cpal::SampleFormat>) -> Self {
        match sample_format.unwrap_or(cpal::SampleFormat::I32) {
            cpal::SampleFormat::I24 => (sample * ((1 << 23) - 1) as f32) as Self,
            cpal::SampleFormat::I32 => (sample * Self::MAX as f32) as Self,
            format => {
                panic!(
                    "Can not call `from_sample` on `i32` with `SampleFormat::{}`! Choose either `I24`, `I32` or `None`. `I32` and `None` produce the same output.",
                    format
                )
            }
        }
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

impl SampleFormatConversion<f32> for i64 {
    #[inline]
    fn to_sample(self, _: Option<cpal::SampleFormat>) -> f32 {
        (self as f64 / Self::MAX as f64) as f32
    }

    #[inline]
    fn to_sample_buf(
        buf: Vec<Self>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<Self>, impl FnMut(Self) -> f32> {
        buf.into_iter().map(|sample| sample.to_sample(None))
    }

    #[inline]
    fn from_sample(sample: f32, _: Option<cpal::SampleFormat>) -> Self {
        (sample as f64 * Self::MAX as f64) as Self
    }

    #[inline]
    fn from_sample_buf(
        buf: Vec<f32>,
        _: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<f32>, impl FnMut(f32) -> Self>
    where
        Self: Sized,
    {
        buf.into_iter()
            .map(|sample| Self::from_sample(sample, None))
    }
}

#[cfg(test)]
mod sample_format_converstion_test {
    use cpal::SampleFormat;

    use crate::audio::traits::SampleFormatConversion;

    #[test]
    fn f32_test() {
        let start: f32 = 1.0;
        let step: f32 = start.to_sample(None);
        let end: f32 = f32::from_sample(step, None);
        assert_eq!(end, start);
        assert_eq!(step, 1.0);
    }

    #[test]
    fn f64_test() {
        let start: f64 = 1.0;
        let step: f32 = start.to_sample(None);
        let end: f64 = f64::from_sample(step, None);

        assert_eq!(end, start);
        assert_eq!(step, 1.0);
    }

    #[test]
    fn u8_test() {
        type BaseType = u8;

        let start: BaseType = 64;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(
            approx_eq_int(start as i128, end as i128, 1),
            "Start: {}; End: {}",
            start,
            end
        );
        assert!(approx_eq(step, -0.498));
    }

    #[test]
    fn u16_test() {
        type BaseType = u16;

        let start: BaseType = 128;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(approx_eq_int(start as i128, end as i128, 1));
        assert!(approx_eq(step, -0.996));
    }

    #[test]
    fn u24_test() {
        type BaseType = u32;

        let start: BaseType = 123456;
        let step: f32 = start.to_sample(Some(SampleFormat::U24));
        let end: BaseType = BaseType::from_sample(step, Some(SampleFormat::U24));

        assert!(approx_eq_int(start as i128, end as i128, 10));
        assert!(approx_eq(step, -0.985));
    }

    #[test]
    fn u32_none_test() {
        type BaseType = u32;

        let start: BaseType = 9342391;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(
            approx_eq_int(start as i128, end as i128, 90),
            "Start: {}; End: {}",
            start,
            end
        );
        assert!(approx_eq(step, -0.995));

        let start2: BaseType = 9342391;
        let step2: f32 = start2.to_sample(Some(SampleFormat::U32));
        let end2: BaseType = BaseType::from_sample(step, Some(SampleFormat::U32));

        assert_eq!(start2, start);
        assert_eq!(step2, step);
        assert_eq!(end2, end);

        assert!(
            approx_eq_int(start2 as i128, end2 as i128, 90),
            "Start2: {}; End2: {}",
            start2,
            end2
        );
        assert!(approx_eq(step, -0.995));
    }

    #[test]
    fn u64_test() {
        type BaseType = u64;

        let start: BaseType = 921343000000000000;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(
            approx_eq_int(start as i128, end as i128, 2915320479745),
            "Start: {}; End: {}; Step: {}",
            start,
            end,
            step
        );
        assert!(approx_eq(step, -0.9));
    }

    #[test]
    fn i8_test() {
        type BaseType = i8;

        let start: BaseType = 126;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(approx_eq_int(start as i128, end as i128, 1));
        assert!(approx_eq(step, 0.992), "Step: {}", step);
    }

    #[test]
    fn i16_test() {
        type BaseType = i16;

        let start: BaseType = 30001;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(approx_eq_int(start as i128, end as i128, 1));
        assert!(approx_eq(step, 0.915));
    }

    #[test]
    fn i24_test() {
        type BaseType = i32;

        let start: BaseType = 120000;
        let step: f32 = start.to_sample(Some(SampleFormat::I24));
        let end: BaseType = BaseType::from_sample(step, Some(SampleFormat::I24));

        assert!(approx_eq_int(start as i128, end as i128, 10));
        assert!(approx_eq(step, 0.014));
    }

    #[test]
    fn i32_test() {
        type BaseType = i32;

        let start: BaseType = 50000690;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(approx_eq_int(start as i128, end as i128, 10));
        assert!(approx_eq(step, 0.023));

        let start2: BaseType = 50000690;
        let step2: f32 = start2.to_sample(Some(SampleFormat::I32));
        let end2: BaseType = BaseType::from_sample(step2, Some(SampleFormat::I32));

        assert_eq!(start2, start);
        assert_eq!(step2, step);
        assert_eq!(end2, end);

        assert!(approx_eq_int(start2 as i128, end as i128, 10));
        assert!(approx_eq(step2, 0.023));
    }

    #[test]
    fn i64_test() {
        type BaseType = i64;

        let start: BaseType = 982182777277666;
        let step: f32 = start.to_sample(None);
        let end: BaseType = BaseType::from_sample(step, None);

        assert!(
            approx_eq_int(start as i128, end as i128, 10000000),
            "Start: {}; End: {}",
            start,
            end
        );
        assert!(approx_eq(step, 0.000));
    }

    fn approx_eq_int(a: i128, b: i128, epsilon: u64) -> bool {
        (a - b).abs() < epsilon as i128
    }

    fn approx_eq(a: f32, b: f32) -> bool {
        const EPSILON: f32 = 0.001;

        (a - b).abs() < EPSILON
    }
}
