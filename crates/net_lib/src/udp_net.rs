use std::net::{SocketAddr, ToSocketAddrs};

use crate::{error as io_error, traits::Bytes, udp_net::inner::Inner};

mod inner;
mod transmission;
mod udp_net_receiver;
mod udp_net_sender;

pub mod error;

pub use transmission::{Receiver, Sender};
pub use udp_net_receiver::UdpNetReceiver;
pub use udp_net_sender::UdpNetSender;

/// A port has 16 bits.
const PORT_LEN: usize = 2;
/// The maximum amount of bytes a UDP packet can carry. This is limited by the 16 bits that is used to store the length of the UDP payload.
const UDP_LENGTH_LEN: usize = 2;
/// The length of the UDP headers checksum field.
const UDP_CHECKSUM_LEN: usize = 2;
/// A UDP header:
///
/// | Field            | Size    |
/// | ---------------- | ------- |
/// | Source Port      | 16 bits |
/// | Destination Port | 16 bits |
/// | Length           | 16 bits |
/// | Checksum         | 16 bits |
///
/// = 64 bits (8 bytes)
const UDP_HEADER_LEN: usize = PORT_LEN + PORT_LEN + UDP_LENGTH_LEN + UDP_CHECKSUM_LEN;
/// IPv4 header size is 20 bytes usually.
/// IPv6 header size is 40 bytes.
const IP_HEADER_LEN: usize = 40;
/// The common MTU (Maximum Transmission Unit) size of networks is 1500 bytes.
const STANDARD_MTU: usize = 1500;
/// This is used to measure if a datagram exceeded the buffer size.
const SAFETY_BYTE: usize = 1;
/// The maximum buffer size the size of the biggest possible datagram.
///
/// More bytes would not make sense.
pub const MAX_BUF_SIZE: usize = u16::MAX as usize;
/// The network safe size that fits in the common MTU (Maximum Transmission Unit) limit.
///
/// This constant respects IPv4 normal headers and IPv6 headers.
pub const SAFE_BUF_SIZE: usize = STANDARD_MTU - UDP_HEADER_LEN - IP_HEADER_LEN + SAFETY_BYTE;

/// A simple UDP networking abstraction.
///
/// ```rust ignore
/// use net_lib::{UdpNet, SAFE_BUF_SIZE};
///
/// let udp_net = UdpNet::<SAFE_BUF_SIZE, Packet>::bind("127.0.0.1:0")?;
/// ```
///
/// If the `BUF_SIZE` is smaller than the maximum datagram size of 65535 (`u16::MAX`) bytes then the last bit is considered a "truncation detection byte".
/// It will be used to detect if a datagram was trucated and is essentially "dead" and will never hold useful data.
/// If you want 1024 bytes of usable buffer then set the `BUF_SIZE` to 1025 bytes.
#[derive(Debug)]
pub struct UdpNet<const BUF_SIZE: usize, P>
where
    P: Bytes,
{
    inner: Inner<P>,
    buf: Box<[u8]>,
}

impl<const BUF_SIZE: usize, P> UdpNet<BUF_SIZE, P>
where
    P: Bytes,
{
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, io_error::IoBindError> {
        Ok(Self {
            inner: Inner::bind(addr)?,
            buf: Box::new([0u8; BUF_SIZE]),
        })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), io_error::IoConnectError> {
        self.inner.connect(addr)
    }

    pub fn local_addr(&self) -> Result<SocketAddr, io_error::IoLocalAddrError> {
        self.inner.local_addr()
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, io_error::IoPeerAddrError> {
        self.inner.peer_addr()
    }

    pub fn split(
        self,
    ) -> Result<(UdpNetSender<BUF_SIZE, P>, UdpNetReceiver<BUF_SIZE, P>), io_error::IoBindError>
    {
        Ok((
            UdpNetSender::new(self.inner.try_clone()?, Box::new([0u8; BUF_SIZE])),
            UdpNetReceiver::new(self.inner, self.buf),
        ))
    }
}

impl<const BUF_SIZE: usize, P> Sender<P> for UdpNet<BUF_SIZE, P>
where
    P: Bytes,
{
    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, &mut self.buf)?;

        Ok(())
    }

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner.send_to(packet, addr, &mut self.buf)?;

        Ok(())
    }
}

impl<const BUF_SIZE: usize, P> Receiver<P> for UdpNet<BUF_SIZE, P>
where
    P: Bytes,
{
    fn peek(&mut self) -> Result<P, error::PeekError> {
        Ok(self.inner.peek(&mut self.buf)?)
    }

    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError> {
        Ok(self.inner.peek_from(&mut self.buf)?)
    }

    fn recv(&mut self) -> Result<P, error::RecvError> {
        Ok(self.inner.recv(&mut self.buf)?)
    }

    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError> {
        Ok(self.inner.recv_from(&mut self.buf)?)
    }
}
