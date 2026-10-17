pub use opus::{Error, ErrorCode};

pub mod decoder;
pub mod encoder;
pub mod opus_slice;

const OPUS_DEFAULT_BITRATE: i32 = 24_000;
const OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE: i32 = 10;
