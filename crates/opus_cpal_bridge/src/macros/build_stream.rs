macro_rules! build_stream {
    (
        $inner:expr,
        $callback:path,
        $args:tt,
        { $($variant:ident),+ $(,)? }
    ) => {
        match $inner.config().sample_format() {
            $(
            ::cpal::SampleFormat::$variant => $inner
                .build_stream::<$crate::macros::build_stream::cpal_sample_format_type!($variant), _, _>(
                    move |buf, info| {
                        $crate::macros::build_stream::call_with!($callback, buf, info, $args)
                    },
                    move |e| ::tracing::error!(
                        concat!("Stream Error ", stringify!($ty), ": {}"),
                        e
                    )
                )
                .expect(concat!("Failed to create new ", stringify!($ty), " stream.")),
            )+
            format => panic!("Unsupported sample format `SampleFormat::{}`!", format),
        }
    };
}
pub(crate) use build_stream;

macro_rules! call_with {
    ($callback:path, $buf:expr, $info:expr, ($($arg:expr),* $(,)?)) => {
        $callback($buf, $info, $($arg),*)
    };
}
pub(crate) use call_with;

macro_rules! cpal_sample_format_type {
    (F32) => {
        f32
    };
    (F64) => {
        f64
    };
    (U8) => {
        u8
    };
    (U16) => {
        u16
    };
    (U24) => {
        ::cpal::U24
    };
    (U32) => {
        u32
    };
    (U64) => {
        u64
    };
    (I8) => {
        i8
    };
    (I16) => {
        i16
    };
    (I24) => {
        ::cpal::I24
    };
    (I32) => {
        i32
    };
    (I64) => {
        i64
    };
    (DsdU8) => {
        u8
    };
    (DsdU16) => {
        u16
    };
    (DsdU32) => {
        u32
    };
}
pub(crate) use cpal_sample_format_type;
