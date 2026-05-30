use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    traits::Bytes,
    udp_net::{Sender, error, inner::Inner},
};

#[derive(Debug)]
pub struct UdpNetSender<P>
where
    P: Bytes,
{
    inner: Inner<P>,
    buf: Vec<u8>,
}

impl<P> UdpNetSender<P>
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

impl<P> Sender<P> for UdpNetSender<P>
where
    P: Bytes,
{
    fn send_bytes(&self, buf: &[u8]) -> io::Result<()> {
        self.inner.send_bytes(buf)
    }

    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, &mut self.buf)?;

        Ok(())
    }

    fn send_bytes_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> io::Result<()> {
        self.inner.send_bytes_to(buf, addr)
    }

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner.send_to(packet, addr, &mut self.buf)?;

        Ok(())
    }
}

buf_ops!(UdpNetSender, buf, false);

socket_options!(UdpNetSender, inner);
