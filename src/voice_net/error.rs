use crate::udp_net;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("UDP Net Error: {0}")]
    UdpNet(#[from] udp_net::error::Error),
}
