use std::{
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    error as io_error,
    traits::bytable::Bytes,
    udp_net::{inner::Inner, udp_net_receiver::UdpNetReceiver, udp_net_sender::UdpNetSender},
};

pub mod error;
pub(self) mod inner;
pub mod udp_net_receiver;
pub mod udp_net_sender;

/// A port has 16 bits.
const PORT_LEN: usize = 2;
/// The maximum amount of bytes a UDP packet can carry. This is limited by the 16 bits that is used to store the length of the UDP payload.
const UDP_LENGTH_LEN: usize = 2;
/// The length of the UDP headers checksum field.
const UDP_CHECKSUM_LEN: usize = 2;
/// A UDP header:
///
/// | Field            | Size     |
/// | ---------------- | -------- |
/// | Source Port      | 16 bytes |
/// | Destination Port | 16 bytes |
/// | Length           | 16 bytes |
/// | Checksum         | 16 bytes |
const UDP_HEADER_LEN: usize = PORT_LEN + PORT_LEN + UDP_LENGTH_LEN + UDP_CHECKSUM_LEN;
/// IPv4 average header size.
const IP_HEADER_LEN: usize = 40;
/// The default receive buffer size.
///
/// `u16::MAX - UDP_HEADER_LEN (8 bytes) - IP_HEADER_LEN (40 bytes)`
#[allow(unused)]
pub const DEFAULT_RECV_BUF_SIZE: usize = u16::MAX as usize - UDP_HEADER_LEN - IP_HEADER_LEN;

/// A simple UDP networking abstraction.
///
/// ```rust
/// use net_lib::{UdpNet, DEFAULT_RECV_BUF_SIZE};
///
/// let udp_net = UdpNet<DEFAULT_RECV_BUF_SIZE>::bind("127.0.0.1:8080")?;
/// ```
#[derive(Debug)]
pub struct UdpNet<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner,
    buf: [u8; BUF_SIZE],

    packet_phantom_data: PhantomData<P>,
}

impl<const BUF_SIZE: usize, P> UdpNet<BUF_SIZE, P>
where
    P: Bytes,
{
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, io_error::BindError> {
        Ok(Self {
            inner: Inner::bind(addr)?,
            buf: [0u8; BUF_SIZE],

            packet_phantom_data: PhantomData,
        })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), io_error::ConnectError> {
        self.inner.connect(addr)
    }

    pub fn send(&mut self, packet: P) -> Result<(), error::SendError> {
        let len = packet.to_bytes(&mut self.buf)?;
        self.inner.send(&self.buf[..len])?;

        Ok(())
    }

    pub fn send_to(&mut self, packet: P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        let len = packet.to_bytes(&mut self.buf)?;
        self.inner.send_to(&self.buf[..len], addr)?;

        Ok(())
    }

    pub fn send_to_all(
        &mut self,
        packet: P,
        addrs: &[impl ToSocketAddrs],
    ) -> Result<(), error::SendError> {
        let len = packet.to_bytes(&mut self.buf)?;
        self.inner.send_to_all(&self.buf[..len], addrs)?;

        Ok(())
    }

    pub fn peek(&mut self) -> Result<P, error::PeekError> {
        let len = self.inner.peek(&mut self.buf)?;
        let packet = P::from_bytes(&self.buf[..len])?;

        Ok(packet)
    }

    pub fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError> {
        let (len, addr) = self.inner.peek_from(&mut self.buf)?;
        let packet = P::from_bytes(&self.buf[..len])?;

        Ok((packet, addr))
    }

    pub fn recv(&mut self) -> Result<P, error::RecvError> {
        let len = self.inner.recv(&mut self.buf)?;
        let packet = P::from_bytes(&self.buf[..len])?;

        Ok(packet)
    }

    pub fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError> {
        let (len, addr) = self.inner.recv_from(&mut self.buf)?;
        let packet = P::from_bytes(&self.buf[..len])?;

        Ok((packet, addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::LocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::PeerAddrError> {
        self.inner.peer_addr()
    }

    pub fn split(
        self,
    ) -> Result<(UdpNetSender<BUF_SIZE>, UdpNetReceiver<BUF_SIZE>), io_error::BindError> {
        Ok((
            UdpNetSender::new(self.inner.try_clone()?, [0u8; BUF_SIZE]),
            UdpNetReceiver::new(self.inner, self.buf),
        ))
    }
}
