macro_rules! io_error_enum {
    ($name:ident, { $($kind:ident), * $(,)? }) => {
        io_error_enum!($name, { $($kind),* }, true);
    };

    ($name:ident, { $($kind:ident), * $(,)? }, true) => {
        io_error_enum!(@enum $name, { $($kind),* });

        impl From<::std::io::Error> for $name {
            fn from(e: ::std::io::Error) -> Self {
                match e.kind() {
                    $( ::std::io::ErrorKind::$kind => $name::$kind(e), )*
                    _ => $name::Other(e),
                }
            }
        }

    };

    ($name:ident, { $($kind:ident), * $(,)? }, false) => {
        io_error_enum!(@enum $name, { $($kind),* });
    };

    (@enum $name:ident, { $($kind:ident), * $(,)? }) => {
        #[derive(Debug, ::thiserror::Error)]
        pub enum $name {
            $( #[error(transparent)] $kind(::std::io::Error), )*
            #[error("Other: {0}")]
            Other(::std::io::Error),
        }
    };
}

io_error_enum!(IoBindError, {
    PermissionDenied,
    AddrInUse,
    InvalidInput,
    AddrNotAvailable,
    InvalidFilename,
    NotFound,
    OutOfMemory,
    ReadOnlyFilesystem,
});

io_error_enum!(IoConnectError, {
    PermissionDenied,
    AddrInUse,
    AddrNotAvailable,
    WouldBlock,
    ConnectionRefused,
    Interrupted,
    NetworkUnreachable,
    TimedOut,
});

io_error_enum!(IoSendError, {
    PermissionDenied,
    WouldBlock,
    ConnectionReset,
    Interrupted,
    InvalidInput,
    OutOfMemory,
    NotConnected,
    BrokenPipe,
}, false);

impl From<std::io::Error> for IoSendError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind::*;

        match e.kind() {
            PermissionDenied => Self::PermissionDenied(e),
            WouldBlock | TimedOut => Self::WouldBlock(e),
            ConnectionReset => Self::ConnectionReset(e),
            Interrupted => Self::Interrupted(e),
            InvalidInput => Self::InvalidInput(e),
            OutOfMemory => Self::OutOfMemory(e),
            NotConnected => Self::NotConnected(e),
            BrokenPipe => Self::BrokenPipe(e),
            _ => Self::Other(e),
        }
    }
}

pub type IoPeekError = IoRecvError;

io_error_enum!(IoRecvError, {
    WouldBlock,
    ConnectionRefused,
    Interrupted,
    InvalidInput,
    OutOfMemory,
    NotConnected,
}, false);

impl From<std::io::Error> for IoRecvError {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind::*;

        match e.kind() {
            WouldBlock | TimedOut => Self::WouldBlock(e),
            ConnectionRefused => Self::ConnectionRefused(e),
            Interrupted => Self::Interrupted(e),
            InvalidInput => Self::InvalidInput(e),
            OutOfMemory => Self::OutOfMemory(e),
            NotConnected => Self::NotConnected(e),
            _ => Self::Other(e),
        }
    }
}

pub type IoLocalAddrError = IoGetSocketNameError;
pub type IoPeerAddrError = IoGetSocketNameError;

io_error_enum!(IoGetSocketNameError, {
    InvalidInput,
});

io_error_enum!(IoGetSocketOption, {
    InvalidInput,
});

io_error_enum!(IoSetSocketOption, {
    InvalidInput,
    OutOfMemory,
});
