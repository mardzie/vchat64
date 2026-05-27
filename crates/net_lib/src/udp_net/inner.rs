use std::net::UdpSocket;

use crate::udp_net::error;

#[derive(Debug)]
pub struct Inner {
    socket: UdpSocket,
}

impl Inner {
    pub fn bind(addr: impl std::net::ToSocketAddrs) -> Result<Self, error::BindError> {
        let socket = UdpSocket::bind(addr)?;

        Ok(Self { socket })
    }

    pub fn try_clone(&self) -> Result<Self, error::BindError> {
        let socket = self.socket.try_clone()?;

        Ok(Inner { socket })
    }
}
