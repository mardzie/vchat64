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

/// This is used to measure if a datagram exceeded the buffer size.
const TRUNCATION_BYTE: usize = 1;

/// IPv4 normally is 20 bytes.
const IPV4_HEADER_SIZE: usize = 20;
const IPV4_OPTIONS_SIZE: usize = 40;
const IPV4_MIN_MTU_SIZE: usize = 576;
/// IPv6 has 40 bytes.
const IPV6_HEADER_SIZE: usize = 40;
const IPV6_MIN_MTU_SIZE: usize = 1280;

const UDP_HEADER_SIZE: usize = 8;
const MAX_DATAGRAM_SIZE: usize = u16::MAX as usize - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;

/// The maximum buffer size. The size of the biggest possible datagram minus Headers.
///
/// Does not include 1 truncation byte because its not needed.
pub const LOOPBACK_BUF_SIZE: usize = MAX_DATAGRAM_SIZE;
pub const INTERNET_BUF_SIZE: usize =
    IPV6_MIN_MTU_SIZE - IPV6_HEADER_SIZE - UDP_HEADER_SIZE + TRUNCATION_BYTE;
pub const INTERNET_BUF_SIZE_LEGACY: usize =
    IPV4_MIN_MTU_SIZE - (IPV4_HEADER_SIZE + IPV4_OPTIONS_SIZE) - UDP_HEADER_SIZE + TRUNCATION_BYTE;

/// A simple UDP networking abstraction.
///
/// ```rust ignore
/// use net_lib::{UdpNet, SAFE_BUF_SIZE};
///
/// let udp_net = UdpNet::<SAFE_BUF_SIZE, Packet>::bind("127.0.0.1:0")?;
/// ```
///
/// If the `BUF_SIZE` is smaller than the maximum datagram size of 65535 (`u16::MAX`) bytes then the last bit is considered a "truncation detection byte".
/// It will be used to detect if a datagram was truncated and is essentially "dead" and will never hold useful data.
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
        assert!(
            BUF_SIZE >= 2,
            "`BUF_SIZE` must be greater than `1`! It needs at least one data bit and one \"truncation detection byte\""
        );

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
            // TODO: Simplify this and the inner workings of `UdpNetSender` and `Inner` with `[0u8; BUF_SIZE - 1]` once `generic_const_exprs` stabilizes.
            // Tracking: https://github.com/rust-lang/rust/issues/76560
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
        self.inner.peek(&mut self.buf)
    }

    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError> {
        self.inner.peek_from(&mut self.buf)
    }

    fn recv(&mut self) -> Result<P, error::RecvError> {
        self.inner.recv(&mut self.buf)
    }

    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError> {
        self.inner.recv_from(&mut self.buf)
    }
}
