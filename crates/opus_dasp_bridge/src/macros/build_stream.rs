macro_rules! build_input_stream {
    ($input:expr, $callback:ident, { $( $variant:ident => $ty:ty ),+ }) => {
        match $input.config().sample_format() {
            $(
            ::cpal::SampleFormat::$variant => $input
                .build_stream(
                    move |buf: &[$ty], info| {
                        $callback(buf, info)
                    },
                    move |e| ::tracing::error!(
                        concat!("Input Stream Error ", stringify!($ty), ": {}"), e
                    )
                ).expect(concat!("Failed to create new ", stringify!($ty), " input stream.")),
            )+
            format => panic!("Unsupported input sample format `SampleFormat::{}`!", format),
        }
    };
}
pub(crate) use build_input_stream;

fn build_input_stream_old(
    input: &mut InputStream,
    input_notify: Sender<InputMessage>,
    mut producer: Caching<Arc<SharedRb<Heap<f32>>>, true, false>,
) {
    match input.config().sample_format() {
        SampleFormat::F32 => input
            .build_stream(
                move |buf: &[f32], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error f32: {}", e),
            )
            .expect("Failed to create new f32 input stream."),
        SampleFormat::F64 => input
            .build_stream(
                move |buf: &[f64], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error f64: {}", e),
            )
            .expect("Failed to create new f64 input stream."),
        SampleFormat::U8 => input
            .build_stream(
                move |buf: &[u8], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error u8: {}", e),
            )
            .expect("Failed to create new u8 input stream."),
        SampleFormat::U16 => input
            .build_stream(
                move |buf: &[u16], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error u16: {}", e),
            )
            .expect("Failed to create new u16 input stream."),
        SampleFormat::U32 => input
            .build_stream(
                move |buf: &[u32], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error u32: {}", e),
            )
            .expect("Failed to create new u32 input stream."),
        SampleFormat::U64 => input
            .build_stream(
                move |buf: &[u64], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error u64: {}", e),
            )
            .expect("Failed to create new u64 input stream."),
        SampleFormat::I8 => input
            .build_stream(
                move |buf: &[i8], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error i8: {}", e),
            )
            .expect("Failed to create new i8 input stream."),
        SampleFormat::I16 => input
            .build_stream(
                move |buf: &[i16], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error i16: {}", e),
            )
            .expect("Failed to create new i16 input stream."),
        SampleFormat::I32 => input
            .build_stream(
                move |buf: &[i32], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error i32: {}", e),
            )
            .expect("Failed to create new i32 input stream."),
        SampleFormat::I64 => input
            .build_stream(
                move |buf: &[i64], info| {
                    Self::input_data_callback(buf, info, &input_notify, &mut producer)
                },
                move |e| tracing::error!("Input Stream Error i64: {}", e),
            )
            .expect("Failed to create new i64 input stream."),
        format => panic!(
            "Unsupported input sample format `SampleFormat::{}`!",
            format
        ),
    }
}

macro_rules! build_output_stream {
    () => {};
}
pub(crate) use build_output_stream;
