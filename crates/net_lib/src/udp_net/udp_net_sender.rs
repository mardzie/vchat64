use std::net::ToSocketAddrs;

use crate::{error, traits::Bytes, udp_net::inner::Inner};

#[derive(Debug)]
pub struct UdpNetSender<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    send_buf: [u8; BUF_SIZE],
}

impl<const BUF_SIZE: usize, P> UdpNetSender<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, send_buf: [u8; BUF_SIZE]) -> Self {
        Self { inner, send_buf }
    }

    pub fn connect(&mut self, addr: impl ToSocketAddrs) -> Result<(), error::ConnectError> {
        self.inner.connect(addr)
    }

    pub fn send(&mut self, packet: P) {}

    pub fn send_to() {}
}
