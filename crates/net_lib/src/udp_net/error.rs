use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BindError {
    #[error("Permissions denied: {0}")]
    PermissionsDenied(io::Error),
    #[error("Address in use: {0}")]
    AddrInUse(io::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(io::Error),
    #[error("Address not available: {0}")]
    AddrNotAvailable(io::Error),
    #[error("Invalid filename: {0}")]
    InvalidFilename(io::Error),
    #[error("Not found: {0}")]
    NotFound(io::Error),
    #[error("Out of memory: {0}")]
    OutOfMemory(io::Error),
    #[error("Read only filesystem: {0}")]
    ReadOnlyFilesystem(io::Error),
}

impl From<std::io::Error> for BindError {
    fn from(e: std::io::Error) -> Self {
        use BindError::*;
        use std::io::ErrorKind;

        match e.kind() {
            ErrorKind::PermissionDenied => PermissionsDenied(e),
            ErrorKind::AddrInUse => AddrInUse(e),
            ErrorKind::InvalidInput => InvalidInput(e),
            ErrorKind::AddrNotAvailable => AddrNotAvailable(e),
            ErrorKind::InvalidFilename => InvalidFilename(e),
            ErrorKind::NotFound => NotFound(e),
            ErrorKind::OutOfMemory => OutOfMemory(e),
            ErrorKind::ReadOnlyFilesystem => ReadOnlyFilesystem(e),

            _ => unreachable!(
                "unexpected error kind for Socket::bind: {} (os error: {:?})",
                e.kind(),
                e.raw_os_error()
            ),
        }
    }
}
