use crate::udp_net::inner::Inner;

#[derive(Debug)]
pub struct UdpNetSender {
    inner: Inner,
}
