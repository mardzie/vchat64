pub mod copy_from_iter_impl;
pub mod normalize_sample_impl;
pub mod sample_origin_impl;

/// Trait for converting some `Sample` into `T` and back.
pub trait NormalizeSample<Normalized>
where
    Self: Sized,
{
    fn normalize(self, sample_format: Option<&cpal::SampleFormat>) -> Normalized;

    fn normalize_buf(
        buf: Vec<Self>,
        sample_format: Option<&cpal::SampleFormat>,
    ) -> impl Iterator<Item = Normalized> {
        buf.into_iter().map(move |raw| raw.normalize(sample_format))
    }

    fn denormalize(sample: Normalized, sample_format: Option<&cpal::SampleFormat>) -> Self;

    fn denormalize_buf(
        buf: Vec<Normalized>,
        sample_format: Option<&cpal::SampleFormat>,
    ) -> impl Iterator<Item = Self> {
        buf.into_iter()
            .map(move |normalized| Self::denormalize(normalized, sample_format))
    }
}

pub trait SampleOrigin {
    /// Returns the center point of this sample type.
    fn origin(sample_format: Option<&cpal::SampleFormat>) -> Self;
}
