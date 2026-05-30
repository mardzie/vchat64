#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error("{0}")]
    Io(#[from] crate::error::IoBindError),
    #[error("{0}")]
    IoLocalAddress(#[from] crate::error::IoLocalAddrError),
}

#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("{0}")]
    Io(#[from] crate::error::IoSendError),
    #[error("{0}")]
    ToBytes(#[from] crate::traits::InsufficientBuffer),
}

pub type PeekError = RecvError;

#[derive(Debug, thiserror::Error)]
pub enum RecvError {
    #[error("{0}")]
    Io(#[from] crate::error::IoRecvError),
    #[error("{0}")]
    FromBytes(#[from] crate::traits::FromByteError),
    #[error("Datagram Truncated Error: The datagram was truncated")]
    DatagramTruncated,
}
