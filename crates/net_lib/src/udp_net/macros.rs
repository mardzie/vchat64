/// Generates an impl for `SocketOptions`.
///
/// # Syntax:
///
/// ```text
/// socket_options!(name: ident, inner: ident);
/// ```
macro_rules! socket_options {
    ($name:ident, $inner:ident) => {
        impl<P> $crate::udp_net::SocketOptions for $name<P>
        where
            P: $crate::traits::Bytes,
        {
            fn read_timeout(&self) -> ::std::io::Result<Option<std::time::Duration>> {
                self.$inner.read_timeout()
            }

            fn set_read_timeout(
                &self,
                dur: Option<::std::time::Duration>,
            ) -> ::std::io::Result<()> {
                self.$inner.set_read_timeout(dur)
            }

            fn write_timeout(&self) -> ::std::io::Result<Option<::std::time::Duration>> {
                self.$inner.write_timeout()
            }

            fn set_write_timeout(
                &self,
                dur: Option<::std::time::Duration>,
            ) -> ::std::io::Result<()> {
                self.$inner.set_write_timeout(dur)
            }

            fn ttl(&self) -> ::std::io::Result<u32> {
                self.$inner.ttl()
            }

            fn set_ttl(&self, ttl: u32) -> ::std::io::Result<()> {
                self.$inner.set_ttl(ttl)
            }

            fn set_nonblocking(&self, nonblocking: bool) -> ::std::io::Result<()> {
                self.$inner.set_nonblocking(nonblocking)
            }
        }
    };
}
pub(crate) use socket_options;

/// Generates an implementation of `BufOps`.
///
/// # Syntax:
///
/// This will implement `BufOps` with truncation.
/// This means the buffer has a truncation detection byte.
/// ```text
/// buf_ops!(name: ident, buf: ident);
/// ```
/// or
/// ```text
/// buf_ops!(name: ident, buf: ident, true);
/// ```
///
/// When appending `false` the macro will generate a impl for `BufOps` without truncation.
/// This means the buffer has no truncation detection byte.
/// ```text
/// buf_ops!(name: ident, buf: ident, false);
/// ```
macro_rules! buf_ops {
    (
        // The name of the struct on which `BufOps` will get implemented.
        $name:ident,
        $buf:ident) => {
        buf_ops!($name, $buf, true);
    };

    ($name:ident, $buf:ident, true) => {
        impl<P> $crate::udp_net::BufOps for $name<P>
        where
            P: $crate::traits::Bytes,
        {
            fn buf_len(&self) -> usize {
                self.$buf.len() - $crate::udp_net::TRUNCATION_BYTE
            }

            /// Resize the buffer to the `new_len` of usable bytes.
            /// This will either expand or shrink the buffer.
            ///
            /// This operation can be expensive.
            /// Only use when necessary.
            fn resize_buf(&mut self, new_len: usize) {
                assert!(new_len > 0);
                $crate::udp_net::resize_buffer(
                    &mut self.$buf,
                    new_len + $crate::udp_net::TRUNCATION_BYTE,
                );
            }
        }
    };

    ($name:ident, $buf:ident, false) => {
        impl<P> $crate::udp_net::BufOps for $name<P>
        where
            P: $crate::traits::Bytes,
        {
            fn buf_len(&self) -> usize {
                self.$buf.len()
            }

            /// Resize the buffer to the `new_len` of usable bytes.
            /// This will either expand or shrink the buffer.
            ///
            /// This operation can be expensive.
            /// Only use when necessary.
            fn resize_buf(&mut self, new_len: usize) {
                assert!(new_len > 0);
                $crate::udp_net::resize_buffer(&mut self.$buf, new_len);
            }
        }
    };
}
pub(crate) use buf_ops;
