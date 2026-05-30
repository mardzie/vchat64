use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    traits::Bytes,
    udp_net::{
        BufOps, Receiver, SocketOptions, TRUNCATION_BYTE, error, inner::Inner, resize_buffer,
    },
};

#[derive(Debug)]
pub struct UdpNetReceiver<P>
where
    P: Bytes,
{
    inner: Inner<P>,
    buf: Vec<u8>,
}

impl<P> UdpNetReceiver<P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: Vec<u8>) -> Self {
        Self { inner, buf }
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> io::Result<()> {
        self.inner.connect(addr)
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr()
    }
}

impl<P> Receiver<P> for UdpNetReceiver<P>
where
    P: Bytes,
{
    fn peek(&mut self) -> Result<P, error::PeekError> {
        self.inner.peek(&mut self.buf)
    }

    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError> {
        self.inner.peek_from(&mut self.buf)
    }

    fn recv(&mut self) -> Result<P, error::RecvError> {
        self.inner.recv(&mut self.buf)
    }

    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError> {
        self.inner.recv_from(&mut self.buf)
    }
}

impl<P> BufOps for UdpNetReceiver<P>
where
    P: Bytes,
{
    fn buf_len(&self) -> usize {
        self.buf.len() - TRUNCATION_BYTE
    }

    /// Resize the buffer to the `new_len` of usable bytes.
    /// This will either expand or shrink the buffer.
    ///
    /// This operation can be expensive.
    /// Only use when necessary.
    fn resize_buf(&mut self, new_len: usize) {
        assert!(new_len > 0);
        resize_buffer(&mut self.buf, new_len + TRUNCATION_BYTE);
    }
}

impl<P> SocketOptions for UdpNetReceiver<P>
where
    P: Bytes,
{
    fn read_timeout(&self) -> io::Result<Option<std::time::Duration>> {
        self.inner.read_timeout()
    }

    fn set_read_timeout(&self, dur: Option<std::time::Duration>) -> io::Result<()> {
        self.inner.set_read_timeout(dur)
    }

    fn write_timeout(&self) -> io::Result<Option<std::time::Duration>> {
        self.inner.write_timeout()
    }

    fn set_write_timeout(&self, dur: Option<std::time::Duration>) -> io::Result<()> {
        self.inner.set_write_timeout(dur)
    }

    fn ttl(&self) -> io::Result<u32> {
        self.inner.ttl()
    }

    fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_ttl(ttl)
    }

    fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }
}
