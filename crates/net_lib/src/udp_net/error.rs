macro_rules! io_error_enum {
    ($name:ident, { $($kind:ident), * $(,)? }) => {
        #[derive(Debug, ::thiserror::Error)]
        pub enum $name {
            $( #[error(transparent)] $kind(::std::io::Error), )*
            #[error("Other: {0}")]
            Other(::std::io::Error),
        }

        impl From<::std::io::Error> for $name {
            fn from(e: ::std::io::Error) -> Self {
                match e.kind() {
                    $( ::std::io::ErrorKind::$kind => $name::$kind(e), )*
                    _ => $name::Other(e),
                }
            }
        }
    };
}

io_error_enum!(BindError, {
    PermissionDenied,
    AddrInUse,
    InvalidInput,
    AddrNotAvailable,
    InvalidFilename,
    NotFound,
    OutOfMemory,
    ReadOnlyFilesystem,
});

io_error_enum!(ConnectError, {
    PermissionDenied,
    AddrInUse,
    AddrNotAvailable,
    WouldBlock,
    ConnectionRefused,
    Interrupted,
    NetworkUnreachable,
    TimedOut,
});

io_error_enum!(SendError, {
    PermissionDenied,
    WouldBlock,
    ConnectionReset,
    Interrupted,
    InvalidInput,
    OutOfMemory,
    NotConnected,
    BrokenPipe,
});

pub type PeekError = RecvError;

io_error_enum!(RecvError, {
    WouldBlock,
    ConnectionRefused,
    Interrupted,
    InvalidInput,
    OutOfMemory,
    NotConnected,
});

pub type LocalAddrError = GetSocketNameError;
pub type PeerAddrError = GetSocketNameError;

io_error_enum!(GetSocketNameError, {
    InvalidInput,
});
