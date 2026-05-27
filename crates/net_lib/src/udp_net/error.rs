use std::io::{self, ErrorKind};

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

#[derive(Debug, Error)]
pub enum ConnectError {
    #[error("Permission denied: {0}")]
    PermissionDenied(io::Error),
    #[error("Addr in use: {0}")]
    AddrInUse(io::Error),
    #[error("Addr not available: {0}")]
    AddrNotAvailable(io::Error),
    #[error("Would block: {0}")]
    WouldBlock(io::Error),
    #[error("Connection refused: {0}")]
    ConnectionRefused(io::Error),
    #[error("Interrupted: {0}")]
    Interrupted(io::Error),
    #[error("Network unreachable: {0}")]
    NetworkUnreachable(io::Error),
    #[error("Timed out: {0}")]
    TimedOut(io::Error),
}

impl From<io::Error> for ConnectError {
    fn from(e: io::Error) -> Self {
        use ConnectError::*;

        match e.kind() {
            ErrorKind::PermissionDenied => PermissionDenied(e),
            ErrorKind::AddrInUse => AddrInUse(e),
            ErrorKind::AddrNotAvailable => AddrNotAvailable(e),
            ErrorKind::WouldBlock => WouldBlock(e),
            ErrorKind::ConnectionRefused => ConnectionRefused(e),
            ErrorKind::Interrupted => Interrupted(e),
            ErrorKind::NetworkUnreachable => NetworkUnreachable(e),
            ErrorKind::TimedOut => TimedOut(e),
            _ => unreachable!(
                "unexpected error kind for Socket::connect: {} (os error {:?})",
                e.kind(),
                e.raw_os_error()
            ),
        }
    }
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error("Permission denied: {0}")]
    PermissionDenied(io::Error),
    #[error("Would block: {0}")]
    WouldBlock(io::Error),
    #[error("Connection reset: {0}")]
    ConnectionReset(io::Error),
    #[error("Interrupted: {0}")]
    Interrupted(io::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(io::Error),
    #[error("Out of memory: {0}")]
    OutOfMemory(io::Error),
    #[error("Not connected: {0}")]
    NotConnected(io::Error),
    #[error("Broken pipe: {0}")]
    BrokenPipe(io::Error),
}

impl From<io::Error> for SendError {
    fn from(e: io::Error) -> Self {
        use SendError::*;

        match e.kind() {
            ErrorKind::PermissionDenied => PermissionDenied(e),
            ErrorKind::WouldBlock => WouldBlock(e),
            ErrorKind::ConnectionReset => ConnectionReset(e),
            ErrorKind::Interrupted => Interrupted(e),
            ErrorKind::InvalidInput => InvalidInput(e),
            ErrorKind::OutOfMemory => OutOfMemory(e),
            ErrorKind::NotConnected => NotConnected(e),
            ErrorKind::BrokenPipe => BrokenPipe(e),
            _ => unreachable!(
                "unexpected error kind for Socket::send: {} (os error: {:?})",
                e.kind(),
                e.raw_os_error()
            ),
        }
    }
}

pub type LocalAddrError = GetSocketNameError;
pub type PeerAddrError = GetSocketNameError;

#[derive(Debug, Error)]
pub enum GetSocketNameError {}

impl From<io::Error> for GetSocketNameError {
    fn from(e: io::Error) -> Self {
        todo!()
    }
}