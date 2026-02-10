use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Voice Send Error: {0}")]
    VoiceSend(#[from] std::sync::mpsc::SendError<Vec<u8>>),
    #[error("Socket closed: {0}")]
    SocketClosed(&'static str),
}
