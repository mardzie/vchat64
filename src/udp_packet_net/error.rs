use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Socket Bind Error: {0}")]
    SocketBind(io::Error),
    #[error("Send Error: {0}")]
    Send(io::Error),
    #[error("Recv Error: {0}")]
    Recv(io::Error),
    #[error("Checksum Mismatch")]
    ChecksumMismatch,
    #[error("Would Block")]
    WouldBlock,
}
