use std::{
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use super::error::SendError;
use crate::{
    error::{self as io_error, BindError, ConnectError, LocalAddrError, PeerAddrError},
    traits::Bytes,
    udp_net::error::{PeekError, RecvError},
};

#[derive(Debug)]
pub struct Inner<P>
where
    P: Bytes,
{
    socket: UdpSocket,

    packet_phantom_data: PhantomData<P>,
}

impl<P> Inner<P>
where
    P: Bytes,
{
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, BindError> {
        let socket = UdpSocket::bind(addr)?;

        Ok(Self {
            socket,
            packet_phantom_data: PhantomData,
        })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), ConnectError> {
        Ok(self.socket.connect(addr)?)
    }

    #[allow(dead_code)]
    pub fn send(&self, packet: &P, buf: &mut [u8]) -> Result<(), SendError> {
        let len = packet.to_bytes(buf)?;
        let _ = self
            .socket
            .send(&buf[..len])
            .map_err(|e| io_error::SendError::from(e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_to(
        &self,
        packet: &P,
        addr: impl ToSocketAddrs,
        buf: &mut [u8],
    ) -> Result<(), SendError> {
        let len = packet.to_bytes(buf)?;
        let _ = self
            .socket
            .send_to(&buf[..len], addr)
            .map_err(|e| io_error::SendError::from(e))?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn send_to_all(
        &self,
        packet: &P,
        addrs: &[impl ToSocketAddrs],
        buf: &mut [u8],
    ) -> Result<(), SendError> {
        let len = packet.to_bytes(buf)?;
        for addr in addrs {
            let _ = self
                .socket
                .send_to(&buf[..len], addr)
                .map_err(|e| io_error::SendError::from(e))?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn peek(&self, buf: &mut [u8]) -> Result<P, PeekError> {
        let len = self
            .socket
            .peek(buf)
            .map_err(|e| io_error::PeekError::from(e))?;
        Ok(P::from_bytes(&buf[..len])?)
    }

    #[allow(dead_code)]
    pub fn peek_from(&self, buf: &mut [u8]) -> Result<(P, SocketAddr), PeekError> {
        let (len, addr) = self
            .socket
            .peek_from(buf)
            .map_err(|e| io_error::PeekError::from(e))?;
        let packet = P::from_bytes(&buf[..len])?;

        Ok((packet, addr))
    }

    #[allow(dead_code)]
    pub fn recv(&self, buf: &mut [u8]) -> Result<P, RecvError> {
        let len = self
            .socket
            .recv(buf)
            .map_err(|e| io_error::RecvError::from(e))?;
        Ok(P::from_bytes(&buf[..len])?)
    }

    #[allow(dead_code)]
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(P, SocketAddr), RecvError> {
        let (len, addr) = self
            .socket
            .recv_from(buf)
            .map_err(|e| io_error::RecvError::from(e))?;
        let packet = P::from_bytes(&buf[..len])?;

        Ok((packet, addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, LocalAddrError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, PeerAddrError> {
        Ok(self.socket.peer_addr()?)
    }

    pub fn try_clone(&self) -> Result<Self, BindError> {
        let socket = self.socket.try_clone()?;

        Ok(Self {
            socket,
            packet_phantom_data: PhantomData,
        })
    }
}
