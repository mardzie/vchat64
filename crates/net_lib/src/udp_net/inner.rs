use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

use crate::error::{
    BindError, ConnectError, LocalAddrError, PeekError, PeerAddrError, RecvError, SendError,
};

#[derive(Debug)]
pub struct Inner {
    socket: UdpSocket,
}

impl Inner {
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, BindError> {
        let socket = UdpSocket::bind(addr)?;

        Ok(Self { socket })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), ConnectError> {
        Ok(self.socket.connect(addr)?)
    }

    #[inline]
    pub fn send(&self, buf: &[u8]) -> Result<usize, SendError> {
        Ok(self.socket.send(buf)?)
    }

    #[inline]
    pub fn send_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> Result<usize, SendError> {
        Ok(self.socket.send_to(buf, addr)?)
    }

    pub fn send_to_all(&self, buf: &[u8], addrs: &[impl ToSocketAddrs]) -> Result<(), SendError> {
        for addr in addrs {
            self.send_to(buf, addr)?;
        }

        Ok(())
    }

    #[inline]
    pub fn peek(&self, buf: &mut [u8]) -> Result<usize, PeekError> {
        Ok(self.socket.peek(buf)?)
    }

    #[inline]
    pub fn peek_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), PeekError> {
        Ok(self.socket.peek_from(buf)?)
    }

    #[inline]
    pub fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError> {
        Ok(self.socket.recv(buf)?)
    }

    #[inline]
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), RecvError> {
        Ok(self.socket.recv_from(buf)?)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, LocalAddrError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, PeerAddrError> {
        Ok(self.socket.peer_addr()?)
    }

    pub fn try_clone(&self) -> Result<Self, BindError> {
        let socket = self.socket.try_clone()?;

        Ok(Inner { socket })
    }
}
