use crate::{traits::Bytes, udp_net::inner::Inner};

#[derive(Debug)]
pub struct UdpNetReceiver<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    recv_buf: [u8; BUF_SIZE],
}

impl<const BUF_SIZE: usize, P> UdpNetReceiver<BUF_SIZE, P>
where
    P: Bytes,
{
    pub(super) fn new(inner: Inner<P>, recv_buf: [u8; BUF_SIZE]) -> Self {
        Self { inner, recv_buf }
    }
}
