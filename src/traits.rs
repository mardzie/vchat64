pub trait SampleFormatConversion<T> {
    /// Converts a sample `self` into `T`.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `SampleFormat` else it will be interpreted as `U32` or `I32`.
    fn convert_sample(self, sample_format: Option<cpal::SampleFormat>) -> T;

    /// Converts a sample list `buf` into an iterator with `T` contents.
    ///
    /// If the sample is `U24` or `I24` you need to specify the `sample_format` else it will be interpreted as `U32` or `I32`.
    fn convert_buf(
        buf: Vec<T>,
        sample_format: Option<cpal::SampleFormat>,
    ) -> std::iter::Map<std::vec::IntoIter<T>, impl FnMut(T) -> T>;
}

pub trait EndiannessConversionDynamic {
    fn to_be(&mut self);

    fn to_le(&mut self);
}
