use crate::udp_net::inner::Inner;

#[derive(Debug)]
pub struct UdpNetReceiver {
    inner: Inner,
}
