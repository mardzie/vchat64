use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    error as io_error,
    traits::Bytes,
    udp_net::{error, inner::Inner, transmission::Receiver},
};

#[derive(Debug)]
pub struct UdpNetReceiver<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    #[allow(dead_code)]
    buf: Box<[u8]>,
}

impl<const BUF_SIZE: usize, P> UdpNetReceiver<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: Box<[u8]>) -> Self {
        Self { inner, buf }
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), io_error::IoConnectError> {
        self.inner.connect(addr)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::IoLocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::IoPeerAddrError> {
        self.inner.peer_addr()
    }
}

impl<const BUF_SIZE: usize, P> Receiver<P> for UdpNetReceiver<BUF_SIZE, P>
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
