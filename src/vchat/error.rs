use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
    #[error("Voice Send Error: {0}")]
    VoiceSendError(#[from] std::sync::mpsc::SendError<Vec<u8>>),
    #[error("Socket closed: {0}")]
    SocketClosedError(&'static str),
}
