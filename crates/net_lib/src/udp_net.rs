use std::net::{SocketAddr, ToSocketAddrs};

use crate::{error as io_error, traits::Bytes, udp_net::inner::Inner};

mod inner;
mod traits;
mod udp_net_receiver;
mod udp_net_sender;

pub mod error;

pub use traits::{BufOps, Receiver, Sender};
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
pub const LOOPBACK_BUF_SIZE: usize = MAX_DATAGRAM_SIZE;
pub const INTERNET_BUF_SIZE: usize = IPV6_MIN_MTU_SIZE - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;
pub const INTERNET_BUF_SIZE_LEGACY: usize =
    IPV4_MIN_MTU_SIZE - (IPV4_HEADER_SIZE + IPV4_OPTIONS_SIZE) - UDP_HEADER_SIZE;

/// A simple UDP networking abstraction.
///
/// ```rust ignore
/// use net_lib::{UdpNet, SAFE_BUF_SIZE};
///
/// let udp_net = UdpNet::<SAFE_BUF_SIZE, Packet>::bind("127.0.0.1:0")?;
/// ```
#[derive(Debug)]
pub struct UdpNet<P>
where
    P: Bytes,
{
    inner: Inner<P>,
    buf: Vec<u8>,
}

impl<P> UdpNet<P>
where
    P: Bytes,
{
    pub fn bind(addr: impl ToSocketAddrs, buf_size: usize) -> Result<Self, io_error::IoBindError> {
        assert!(
            buf_size >= 1,
            "`buf_size` must be greater than `1`! It needs at least one data bit and one \"truncation detection byte\""
        );

        Ok(Self {
            inner: Inner::bind(addr)?,
            buf: vec![0u8; buf_size + TRUNCATION_BYTE],
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

    pub fn split(self) -> Result<(UdpNetSender<P>, UdpNetReceiver<P>), io_error::IoBindError> {
        Ok((
            UdpNetSender::new(
                self.inner.try_clone()?,
                vec![0u8; self.buf.len() - TRUNCATION_BYTE],
            ),
            UdpNetReceiver::new(self.inner, self.buf),
        ))
    }

    fn usable_buf(buf: &mut [u8]) -> &mut [u8] {
        let len = buf.len() - TRUNCATION_BYTE;
        &mut buf[..len]
    }
}

impl<P> Sender<P> for UdpNet<P>
where
    P: Bytes,
{
    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, Self::usable_buf(&mut self.buf))?;

        Ok(())
    }

    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner
            .send_to(packet, addr, Self::usable_buf(&mut self.buf))?;

        Ok(())
    }
}

impl<P> Receiver<P> for UdpNet<P>
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

impl<P> BufOps for UdpNet<P>
where
    P: Bytes,
{
    fn buf_len(&self) -> usize {
        self.buf.len() - TRUNCATION_BYTE
    }

    /// Resize the buffer to the `new_len` of usable bytes.
    /// This will either expand or shrink the buffer.
    ///
    /// This operation can be expensive.
    /// Only use when necessary.
    fn resize_buf(&mut self, new_len: usize) {
        assert!(new_len > 0);
        resize_buffer(&mut self.buf, new_len + TRUNCATION_BYTE);
    }
}

fn resize_buffer(buf: &mut Vec<u8>, new_len: usize) {
    buf.resize(new_len, 0);
    buf.shrink_to_fit();
}
