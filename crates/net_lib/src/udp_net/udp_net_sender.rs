use crate::udp_net::inner::Inner;

#[derive(Debug)]
pub struct UdpNetSender<const BUF_SIZE: usize> {
    inner: Inner,
    send_buf: [u8; BUF_SIZE],
}

impl<const BUF_SIZE: usize> UdpNetSender<BUF_SIZE> {
    pub(super) fn new(inner: Inner, send_buf: [u8; BUF_SIZE]) -> Self {
        Self { inner, send_buf }
    }
}
