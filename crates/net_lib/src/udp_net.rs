//! UDP related types and traits.
//!
//! This module provides UDP networking types.

use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::udp_net::{
    inner::Inner,
    macros::{buf_ops, socket_options},
};

mod inner;
mod macros;
mod traits;
mod udp_net_receiver;
mod udp_net_sender;

pub mod error;

use serde::{Serialize, de::DeserializeOwned};
pub use traits::{BufOps, Receiver, Sender, SocketOptions};
pub use udp_net_receiver::UdpNetReceiver;
pub use udp_net_sender::UdpNetSender;

/// This is used to measure if a datagram exceeded the buffer size.
const TRUNCATION_BYTE: usize = 1;

/// IPv4 normally is 20 bytes.
const IPV4_HEADER_SIZE: usize = 20;
const IPV4_MIN_MTU_SIZE: usize = 576;
/// IPv6 has 40 bytes.
const IPV6_HEADER_SIZE: usize = 40;
const IPV6_MIN_MTU_SIZE: usize = 1280;

const UDP_HEADER_SIZE: usize = 8;
const MAX_IPV4_DATAGRAM_SIZE: usize = u16::MAX as usize - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;
const MAX_IPV6_DATAGRAM_SIZE: usize = u16::MAX as usize - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;

/// The maximum *reasonable* buffer size. The size of the biggest possible payload for **IPv4** subtracting headers.
///
/// `65535 bytes - IPv4_HEADER (20 bytes) - UDP_HEADER (8 bytes)`
pub const LOOPBACK_BUF_SIZE: usize = MAX_IPV4_DATAGRAM_SIZE;
/// The typical maximum payload size for **IPv6**.
///
/// `IPv6_MTU (1280 bytes) - IPv6_HEADER (40 bytes) - UDP_HEADER (8 bytes)`
pub const INTERNET_BUF_SIZE: usize = IPV6_MIN_MTU_SIZE - IPV6_HEADER_SIZE - UDP_HEADER_SIZE;
/// The typical maximum payload size for **IPv4**.
///
/// `IPv4_MTU (576 bytes) - IPv4_HEADER (20 bytes) - UDP_HEADER (8 bytes)`
pub const INTERNET_BUF_SIZE_LEGACY: usize = IPV4_MIN_MTU_SIZE - IPV4_HEADER_SIZE - UDP_HEADER_SIZE;

/// A simple thin UDP networking abstraction.
///
/// ```rust ignore
/// use net_lib::udp_net::{UdpNet, INTERNET_BUF_SIZE};
///
/// let udp_net = UdpNet::<Packet>::bind("127.0.0.1:0", INTERNET_BUF_SIZE)?;
/// ```
#[derive(Debug)]
pub struct UdpNet<P>
where
    P: Serialize + DeserializeOwned,
{
    inner: Inner<P>,
    buf: Vec<u8>,
}

impl<P> UdpNet<P>
where
    P: Serialize + DeserializeOwned,
{
    pub fn bind(addr: impl ToSocketAddrs, buf_size: usize) -> io::Result<Self> {
        assert!(
            buf_size > 0,
            "`buf_size` must be greater than `0`! It needs at least one data bit and one \"truncation detection byte\""
        );

        Ok(Self {
            inner: Inner::bind(addr)?,
            buf: vec![0u8; buf_size + TRUNCATION_BYTE],
        })
    }

    /// Connects this socket to and remote address.
    ///
    /// [`UdpNet::send()`], [`UdpNet::peek()`] and [`UdpNet::recv()`] will fail when connect was not called beforehand [`UdpNet::connect()`].
    pub fn connect(&self, addr: impl ToSocketAddrs) -> io::Result<()> {
        self.inner.connect(addr)
    }

    /// Returns the local sockets socket address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }

    /// Returns the socket address of the remote peer this socket was connected to.
    ///
    /// [`Inner::connect()`] will connect the socket to a remote address.
    /// This method will return an [`std::io::ErrorKind::NotConnected`] error if the socket is not connected.
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr()
    }

    /// Splits the socket.
    ///
    /// Both returned `UdpNetSender` and `UdpNetReceiver` will share one os socket but have two different handles and two different buffers.
    /// One extra buffer will be allocated on this call.
    pub fn split(self) -> io::Result<(UdpNetSender<P>, UdpNetReceiver<P>)> {
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
    P: Serialize + DeserializeOwned,
{
    /// Send bytes directly to the connected address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `Deserialize` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to the connected address.
    ///
    /// [`UdpNet::connect()`] will connect the socket to a remote address. This method will fail if the socket is not connected.
    fn send_bytes(&self, buf: &[u8]) -> io::Result<()> {
        self.inner.send_bytes(buf)
    }

    /// Send a `P` to the connected address.
    ///
    /// [`UdpNet::connect()`] will connect the socket to a remote address. This method will fail if the socket is not connected.
    fn send(&mut self, packet: &P) -> Result<(), error::SendError> {
        self.inner.send(packet, Self::usable_buf(&mut self.buf))?;

        Ok(())
    }

    /// Send bytes directly to the address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `Deserialize` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to one or more addresses.
    fn send_bytes_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> io::Result<()> {
        self.inner.send_bytes_to(buf, addr)
    }

    /// Send a `P` to an address.
    fn send_to(&mut self, packet: &P, addr: impl ToSocketAddrs) -> Result<(), error::SendError> {
        self.inner
            .send_to(packet, addr, Self::usable_buf(&mut self.buf))?;

        Ok(())
    }
}

impl<P> Receiver<P> for UdpNet<P>
where
    P: Serialize + DeserializeOwned,
{
    /// Peek a `P` from the connected address.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    ///
    /// [`UdpNet::connect()`] will connect the socket to a remote address. This method will fail if the socket is not connected.
    fn peek(&mut self) -> Result<P, error::PeekError> {
        self.inner.peek(&mut self.buf)
    }

    /// Peek a `P` from the socket.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    fn peek_from(&mut self) -> Result<(P, SocketAddr), error::PeekError> {
        self.inner.peek_from(&mut self.buf)
    }

    /// Receive a `P` from the connected address.
    ///
    /// [`UdpNet::connect()`] will connect the socket to a remote address. This method will fail if the socket is not connected.
    fn recv(&mut self) -> Result<P, error::RecvError> {
        self.inner.recv(&mut self.buf)
    }

    /// Receive a `P` from the socket.
    fn recv_from(&mut self) -> Result<(P, SocketAddr), error::RecvError> {
        self.inner.recv_from(&mut self.buf)
    }
}

buf_ops!(UdpNet, buf);

socket_options!(UdpNet, inner);

fn resize_buffer(buf: &mut Vec<u8>, new_len: usize) {
    buf.resize(new_len, 0);
    buf.shrink_to_fit();
}
