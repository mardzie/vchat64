use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

use crate::{traits::Bytes, udp_net::error};

pub trait Sender<P: Bytes> {
    /// Send bytes directly to the connected address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `FromBytes` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to the connected address.
    fn send_bytes(&self, buf: &[u8]) -> io::Result<()>;

    /// Send a `P` to the connected address.
    fn send(&mut self, packet: &P) -> Result<(), error::SendError>;

    /// Send bytes directly to the address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `FromBytes` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to one or more addresses.
    fn send_bytes_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> io::Result<()>;

    /// Send a `P` to an address.
    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError>;
}

pub trait Receiver<P: Bytes> {
    /// Peek a `P` from the connected address.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    fn peek(&mut self) -> Result<P, error::PeekError>;

    /// Peek a `P` from an address.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError>;

    /// Receive a `P` from the connected address.
    fn recv(&mut self) -> Result<P, error::RecvError>;

    /// Receive a `P` from an address.
    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError>;
}

pub trait SocketOptions {
    fn read_timeout(&self) -> io::Result<Option<std::time::Duration>>;

    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;

    fn write_timeout(&self) -> io::Result<Option<Duration>>;

    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()>;

    fn ttl(&self) -> io::Result<u32>;

    fn set_ttl(&self, ttl: u32) -> io::Result<()>;

    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()>;
}

pub trait BufOps {
    /// Returns the current buffer length.
    fn buf_len(&self) -> usize;

    /// Resizes the buffer to the set length.
    fn resize_buf(&mut self, new_len: usize);
}
