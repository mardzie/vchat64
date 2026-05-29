use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    error::{self as io_error, ConnectError},
    traits::Bytes,
    udp_net::{error, inner::Inner, transmission::Sender},
};

#[derive(Debug)]
pub struct UdpNetSender<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    #[allow(dead_code)]
    buf: Box<[u8]>,
}

impl<const BUF_SIZE: usize, P> UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: Box<[u8]>) -> Self {
        Self { inner, buf }
    }

    pub fn connect(&mut self, addr: impl ToSocketAddrs) -> Result<(), ConnectError> {
        self.inner.connect(addr)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::LocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::PeerAddrError> {
        self.inner.peer_addr()
    }
}

impl<const BUF_SIZE: usize, P> Sender<P> for UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, &mut self.buf)?;

        Ok(())
    }

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner.send_to(packet, addr, &mut self.buf)?;

        Ok(())
    }

    fn send_to_all(
        &mut self,
        packet: &P,
        addrs: &[impl ToSocketAddrs],
    ) -> Result<(), error::SendError> {
        self.inner.send_to_all(packet, addrs, &mut self.buf)?;

        Ok(())
    }
}
