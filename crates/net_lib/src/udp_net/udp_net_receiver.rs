use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    traits::Bytes,
    udp_net::{
        Receiver, error,
        inner::Inner,
        macros::{buf_ops, socket_options},
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

buf_ops!(UdpNetReceiver, buf);

socket_options!(UdpNetReceiver, inner);
