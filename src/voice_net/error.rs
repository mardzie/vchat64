use std::io;

use crate::udp_packet_net;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("UDP Net Error: {0}")]
    UdpNet(#[from] udp_packet_net::error::Error),
    #[error("Send Error: {0}")]
    Send(io::Error),
    #[error("Would Block")]
    WouldBlock,
}
