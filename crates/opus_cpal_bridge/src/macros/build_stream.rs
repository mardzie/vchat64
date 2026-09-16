macro_rules! build_input_stream {
    (
        $input:expr,
        $callback:path,
        $args:tt,
        { $($variant:ident),+ $(,)? }
    ) => {
        match $input.config().sample_format() {
            $(
            ::cpal::SampleFormat::$variant => $input
                .build_input_stream(
                    move |buf: &[$crate::macros::build_stream::cpal_sample_format_type!($variant)], info| {
                        $crate::macros::build_stream::call_with!($callback, buf, info, $args)
                    },
                    move |e| ::tracing::error!(
                        concat!("Input Stream Error ", stringify!($ty), ": {}"), e
                    )
                )
                .expect(concat!("Failed to create new ", stringify!($ty), " input stream.")),
            )+
            format => panic!("Unsupported input sample format `SampleFormat::{}`!", format),
        }
    };
}
pub(crate) use build_input_stream;

macro_rules! build_output_stream {
    (
        $output:expr,
        $callback:path,
        $args:tt,
        { $($variant:ident),+ $(,)? }
    ) => {
        match $output.config().sample_format() {
            $(
            ::cpal::SampleFormat::$variant => $output
                .build_output_stream(
                    move |buf: &mut [$crate::macros::build_stream::cpal_sample_format_type!($variant)], info| {
                        $crate::macros::build_stream::call_with!($callback, buf, info, $args)
                    },
                    move |e| ::tracing::error!(
                        concat!("Output Stream Error ", stringify!($ty), ": {}"), e
                    )
                )
                .expect(concat!("Failed to create new ", stringify!($ty), " output stream.")),
            )+
            format => panic!("Unsupported output sample format `SampleFormat::{}`!", format),
        }
    };
}
pub(crate) use build_output_stream;

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
        u32
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
        i32
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
