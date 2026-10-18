pub use opus::{Error, ErrorCode};

pub mod decoder;
pub mod encoder;
pub mod frame_duration;
pub mod opus_slice;

const OPUS_DEFAULT_BITRATE: u32 = 24_000;
const OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE: u8 = 10;
// TODO: Check for actual percentage range that is supported.
const _: () = assert!(OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE <= 100);
