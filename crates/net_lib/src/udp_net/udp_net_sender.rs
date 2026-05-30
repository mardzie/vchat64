use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    error::{self as io_error, IoConnectError},
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
    /// The size of the usable buffer.
    ///
    /// The last bit is used in the receiver to check for truncation.
    /// Here its useless until `generic_const_exprs` stabilized.
    /// Then the buffer can be reduced by 1 and this logic to ignore the extra byte removed.
    /// Tracking: https://github.com/rust-lang/rust/issues/76560
    buf_size: usize,
}

impl<const BUF_SIZE: usize, P> UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: Box<[u8]>) -> Self {
        let buf_size = buf.len() - 1;
        Self {
            inner,
            buf,
            buf_size,
        }
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), IoConnectError> {
        self.inner.connect(addr)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::IoLocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::IoPeerAddrError> {
        self.inner.peer_addr()
    }
}

impl<const BUF_SIZE: usize, P> Sender<P> for UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, &mut self.buf[..self.buf_size])?;

        Ok(())
    }

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner
            .send_to(packet, addr, &mut self.buf[..self.buf_size])?;

        Ok(())
    }
}
