use std::{
    net::{SocketAddr, ToSocketAddrs},
    time::Duration,
};

use crate::{
    error::{IoGetSocketOption, IoSetSocketOption},
    traits::Bytes,
    udp_net::error,
};

#[allow(dead_code)]
pub trait Sender<P: Bytes> {
    fn send(&mut self, packet: &P) -> Result<(), error::SendError>;

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError>;
}

#[allow(dead_code)]
pub trait Receiver<P: Bytes> {
    fn peek(&mut self) -> Result<P, error::PeekError>;

    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError>;

    fn recv(&mut self) -> Result<P, error::RecvError>;

    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError>;
}

pub trait SocketOptions {
    fn read_timeout(&self) -> Result<Option<std::time::Duration>, IoGetSocketOption>;

    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<(), IoSetSocketOption>;

    fn write_timeout(&self) -> Result<Option<Duration>, IoGetSocketOption>;

    fn set_write_timeout(&self, dur: Option<Duration>) -> Result<(), IoSetSocketOption>;

    fn ttl(&self) -> Result<u32, IoGetSocketOption>;

    fn set_ttl(&self, ttl: u32) -> Result<(), IoSetSocketOption>;

    fn set_nonblocking(&self, nonblocking: bool) -> Result<(), IoSetSocketOption>;
}

pub trait BufOps {
    fn buf_len(&self) -> usize;

    /// Resizes the buffer to the set length.
    fn resize_buf(&mut self, new_len: usize);
}
