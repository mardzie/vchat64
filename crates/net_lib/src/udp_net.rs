use crate::udp_net::inner::Inner;

pub mod error;
pub(self) mod inner;
pub mod udp_net_receiver;
pub mod udp_net_sender;

#[derive(Debug)]
pub struct UdpNet {
    inner: Inner,
}

impl UdpNet {
    pub fn bind(addr: impl std::net::ToSocketAddrs) -> Result<Self, error::BindError> {
        let inner = Inner::bind(addr)?;

        Ok(Self { inner })
    }

    pub fn split(&self) {}
}
