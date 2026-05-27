use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::udp_net::error::{self, ConnectError, SendError};

#[derive(Debug)]
pub struct Inner {
    socket: UdpSocket,
}

impl Inner {
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, error::BindError> {
        let socket = UdpSocket::bind(addr)?;

        Ok(Self { socket })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), ConnectError> {
        Ok(self.socket.connect(addr)?)
    }

    pub fn send(&self, buf: &[u8]) -> Result<usize, SendError> {
        Ok(self.socket.send(buf)?)
    }

    pub fn send_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> Result<usize, SendError> {
        Ok(self.socket.send_to(buf, addr)?)
    }

    pub fn send_to_all(
        &self,
        buf: &[u8],
        addrs: &[impl ToSocketAddrs],
    ) -> Result<(), Vec<SendError>> {
        let mut error_vec = Vec::new();
        for addr in addrs {
            if let Err(e) = self.send_to(buf, addr) {
                error_vec.push(e);
            }
        }

        if error_vec.is_empty() {
            Ok(())
        } else {
            Err(error_vec)
        }
    }

    pub fn local_addr(&self) -> Result<SocketAddr, > {
        self.socket.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, > {
        self.socket.peer_addr()
    }

    pub fn try_clone(&self) -> Result<Self, error::BindError> {
        let socket = self.socket.try_clone()?;

        Ok(Inner { socket })
    }
}
