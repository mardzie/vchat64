pub use opus::{Error, ErrorCode};

pub mod decoder;
pub mod encoder;
pub mod error;
pub mod frame_size;

pub const OPUS_DEFAULT_BITRATE: u32 = 24_000;
pub const OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE: u8 = 10;
// Packet loss percentage is 0 - 100 inclusive https://opus-codec.org/docs/opus_api-1.6/group__opus__encoderctls.html#gafda1c951dea919ba54432cd03827f1a9
const _: () = assert!(OPUS_DEFAULT_PACKET_LOSS_PERCENTAGE <= 100);
