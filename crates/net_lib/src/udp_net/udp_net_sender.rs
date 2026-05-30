use std::net::{SocketAddr, ToSocketAddrs};

use crate::{
    error::{self as io_error, IoConnectError},
    traits::Bytes,
    udp_net::{BufOps, Sender, error, inner::Inner, resize_buffer},
};

#[derive(Debug)]
pub struct UdpNetSender<P>
where
    P: Bytes,
{
    inner: Inner<P>,
    #[allow(dead_code)]
    buf: Vec<u8>,
}

impl<P> UdpNetSender<P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, buf: Vec<u8>) -> Self {
        Self { inner, buf }
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

impl<P> Sender<P> for UdpNetSender<P>
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
}

impl<P> BufOps for UdpNetSender<P>
where
    P: Bytes,
{
    fn buf_len(&self) -> usize {
        self.buf.len()
    }
    
    /// Resize the buffer to the new length.
    /// This will either expand or shrink the buffer.
    ///
    /// This operation can be expensive.
    /// Only use when necessary.
    fn resize_buf(&mut self, new_len: usize) {
        assert!(new_len > 0);
        resize_buffer(&mut self.buf, new_len);
    }
}
