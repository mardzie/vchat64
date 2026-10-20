pub use opus::{Error, ErrorCode};

pub mod decoder;
pub mod encoder;
pub mod error;
pub mod frame_size;

pub const OPUS_DEFAULT_BITRATE: u32 = 24_000;
pub const OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE: u8 = 10;
// TODO: Check for actual percentage range that is supported.
const _: () = assert!(OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE <= 100);
