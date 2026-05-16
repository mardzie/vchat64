use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SendError {
    #[error("Send IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Would Block")]
    WouldBlock,
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error("Recv IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Checksum Mismatch")]
    ChecksumMismatch,
}
