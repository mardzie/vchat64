use crate::udp_net::inner::Inner;

#[derive(Debug)]
pub struct UdpNetReceiver<const BUF_SIZE: usize> {
    inner: Inner,
    recv_buf: [u8; BUF_SIZE],
}

impl<const BUF_SIZE: usize> UdpNetReceiver<BUF_SIZE> {
    pub(super) fn new(inner: Inner, recv_buf: [u8; BUF_SIZE]) -> Self {
        Self { inner, recv_buf }
    }
}
