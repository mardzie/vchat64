//! Error types for the `udp_net` module.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Serialize(#[from] postcard::Error),
}

pub type PeekError = RecvError;

#[derive(Debug, thiserror::Error)]
pub enum RecvError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Deserialize(#[from] postcard::Error),
    #[error("Datagram Truncated Error: The datagram was truncated")]
    DatagramTruncated,
}
