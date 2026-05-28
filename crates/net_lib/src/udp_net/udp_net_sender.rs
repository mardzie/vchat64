use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    error::{self as io_error, ConnectError},
    traits::Bytes,
    udp_net::{error, inner::Inner},
};

#[derive(Debug)]
pub struct UdpNetSender<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    buf: [u8; BUF_SIZE],
}

impl<const BUF_SIZE: usize, P> UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: [u8; BUF_SIZE]) -> Self {
        Self { inner, buf }
    }

    pub fn connect(&mut self, addr: impl ToSocketAddrs) -> Result<(), ConnectError> {
        self.inner.connect(addr)
    }

    pub fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, &mut self.buf)?;

        Ok(())
    }

    pub fn send_to(
        &mut self,
        packet: &P,
        addr: impl ToSocketAddrs,
    ) -> Result<(), error::SendError> {
        self.inner.send_to(packet, addr, &mut self.buf)?;

        Ok(())
    }

    pub fn send_to_all(
        &mut self,
        packet: &P,
        addrs: &[impl ToSocketAddrs],
    ) -> Result<(), error::SendError> {
        self.inner.send_to_all(packet, addrs, &mut self.buf)?;

        Ok(())
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::LocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::PeerAddrError> {
        self.inner.peer_addr()
    }
}
