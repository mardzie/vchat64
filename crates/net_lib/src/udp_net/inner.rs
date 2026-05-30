use std::{
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    time::Duration,
};

use super::error::SendError;
use crate::{
    error::{
        self as io_error, IoBindError, IoConnectError, IoGetSocketOption, IoLocalAddrError,
        IoPeerAddrError, IoSendError, IoSetSocketOption,
    },
    traits::Bytes,
    udp_net::{
        MAX_IPV4_DATAGRAM_SIZE, MAX_IPV6_DATAGRAM_SIZE, SocketOptions,
        error::{BindError, PeekError, RecvError},
    },
};

#[derive(Debug)]
pub struct Inner<P>
where
    P: Bytes,
{
    socket: UdpSocket,
    addr_type: AddrType,

    packet_phantom_data: PhantomData<P>,
}

impl<P> Inner<P>
where
    P: Bytes,
{
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, BindError> {
        let socket = UdpSocket::bind(addr).map_err(IoBindError::from)?;
        let addr = socket.local_addr().map_err(IoBindError::from)?;
        let addr_type = AddrType::from(addr);

        Ok(Self {
            socket,
            addr_type,

            packet_phantom_data: PhantomData,
        })
    }

    pub fn connect(&self, addr: impl ToSocketAddrs) -> Result<(), IoConnectError> {
        Ok(self.socket.connect(addr)?)
    }

    /// Send bytes directly to the connected address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `FromBytes` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to the connected address.
    pub fn send_bytes(&self, buf: &[u8]) -> Result<(), IoSendError> {
        self.socket.send(buf)?;

        Ok(())
    }

    /// Send a `P` to the connected address.
    pub fn send(&self, packet: &P, buf: &mut [u8]) -> Result<(), SendError> {
        let len = packet.to_bytes(buf)?;
        self.send_bytes(&buf[..len])?;

        Ok(())
    }

    /// Send bytes directly to the address.
    ///
    /// The `buf`s content must be able to be turned back into `P` with the `FromBytes` trait or the receiver will get an error.
    ///
    /// This skips the to_bytes call and is useful if the same packet gets sent multiple times to one or more addresses.
    pub fn send_bytes_to(&self, buf: &[u8], addr: impl ToSocketAddrs) -> Result<(), IoSendError> {
        self.socket.send_to(buf, addr)?;

        Ok(())
    }

    /// Send a `P` to an address.
    pub fn send_to(
        &self,
        packet: &P,
        addr: impl ToSocketAddrs,
        buf: &mut [u8],
    ) -> Result<(), SendError> {
        let len = packet.to_bytes(buf)?;
        self.send_bytes_to(&buf[..len], addr)?;

        Ok(())
    }

    /// Peek a `P` from the connected address.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    pub fn peek(&self, buf: &mut [u8]) -> Result<P, PeekError> {
        let len = self.socket.peek(buf).map_err(io_error::IoPeekError::from)?;
        Self::check_for_truncation(&self.addr_type, buf, len)?;
        Ok(P::from_bytes(&buf[..len])?)
    }

    /// Peek a `P` from an address.
    ///
    /// This will not remove the `P` from the sockets received datagrams.
    pub fn peek_from(&self, buf: &mut [u8]) -> Result<(P, SocketAddr), PeekError> {
        let (len, addr) = self
            .socket
            .peek_from(buf)
            .map_err(io_error::IoPeekError::from)?;
        Self::check_for_truncation(&self.addr_type, buf, len)?;
        let packet = P::from_bytes(&buf[..len])?;

        Ok((packet, addr))
    }

    /// Receive a `P` from the connected address.
    pub fn recv(&self, buf: &mut [u8]) -> Result<P, RecvError> {
        let len = self.socket.recv(buf).map_err(io_error::IoRecvError::from)?;
        Self::check_for_truncation(&self.addr_type, buf, len)?;
        Ok(P::from_bytes(&buf[..len])?)
    }

    /// Receive a `P` from an address.
    pub fn recv_from(&self, buf: &mut [u8]) -> Result<(P, SocketAddr), RecvError> {
        let (len, addr) = self
            .socket
            .recv_from(buf)
            .map_err(io_error::IoRecvError::from)?;
        Self::check_for_truncation(&self.addr_type, buf, len)?;
        let packet = P::from_bytes(&buf[..len])?;

        Ok((packet, addr))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, IoLocalAddrError> {
        Ok(self.socket.local_addr()?)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, IoPeerAddrError> {
        Ok(self.socket.peer_addr()?)
    }

    pub fn try_clone(&self) -> Result<Self, IoBindError> {
        let socket = self.socket.try_clone()?;

        Ok(Self {
            socket,
            addr_type: self.addr_type,

            packet_phantom_data: PhantomData,
        })
    }

    /// Uses the last byte as an indicator that the datagram was truncated.
    /// This does not hold true when the buffer is the `MAX_DATAGRAM_SIZE`
    fn check_for_truncation(addr_type: &AddrType, buf: &[u8], len: usize) -> Result<(), RecvError> {
        // `max_datagram_size` is the maximum size a datagram can have.
        // When the `buf` is that size the truncation check gets disabled and the last byte can be used as a data byte.
        if buf.len() < addr_type.max_datagram_size() && len == buf.len() {
            return Err(PeekError::DatagramTruncated);
        }

        Ok(())
    }
}

impl<P> SocketOptions for Inner<P>
where
    P: Bytes,
{
    fn read_timeout(&self) -> Result<Option<std::time::Duration>, IoGetSocketOption> {
        Ok(self.socket.read_timeout()?)
    }

    fn set_read_timeout(&self, dur: Option<Duration>) -> Result<(), IoSetSocketOption> {
        Ok(self.socket.set_read_timeout(dur)?)
    }

    fn write_timeout(&self) -> Result<Option<Duration>, IoGetSocketOption> {
        Ok(self.socket.write_timeout()?)
    }

    fn set_write_timeout(&self, dur: Option<Duration>) -> Result<(), IoSetSocketOption> {
        Ok(self.socket.set_write_timeout(dur)?)
    }

    fn ttl(&self) -> Result<u32, IoGetSocketOption> {
        Ok(self.socket.ttl()?)
    }

    fn set_ttl(&self, ttl: u32) -> Result<(), IoSetSocketOption> {
        Ok(self.socket.set_ttl(ttl)?)
    }

    fn set_nonblocking(&self, nonblocking: bool) -> Result<(), IoSetSocketOption> {
        Ok(self.socket.set_nonblocking(nonblocking)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AddrType {
    IPv4,
    IPv6,
}

impl AddrType {
    pub fn max_datagram_size(&self) -> usize {
        match self {
            Self::IPv4 => MAX_IPV4_DATAGRAM_SIZE,
            Self::IPv6 => MAX_IPV6_DATAGRAM_SIZE,
        }
    }
}

impl From<SocketAddr> for AddrType {
    fn from(addr: SocketAddr) -> Self {
        if addr.is_ipv4() {
            Self::IPv4
        } else if addr.is_ipv6() {
            Self::IPv6
        } else {
            panic!("Unknown socket address type: {}", addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::{FromByteError, FromBytes, InsufficientBuffer, ToBytes};

    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Packet {
        Option1,
        Option2,
    }

    impl ToBytes for Packet {
        fn to_bytes(&self, buf: &mut [u8]) -> Result<usize, crate::traits::InsufficientBuffer> {
            if buf.len() < 1 {
                return Err(InsufficientBuffer);
            }

            buf[0] = self.clone() as u8;
            Ok(1)
        }
    }

    impl FromBytes for Packet {
        fn from_bytes(buf: &[u8]) -> Result<Self, crate::traits::FromByteError> {
            if buf.len() != 1 {
                return Err(FromByteError::UnexpectedEOF {
                    needed: 1,
                    available: buf.len(),
                    desc: "Packet need 1 byte for decoding".to_string(),
                });
            }

            match buf[0] {
                0 => Ok(Self::Option1),
                1 => Ok(Self::Option2),
                _ => Err(crate::traits::FromByteError::InvalidData {
                    offset: 0,
                    desc: "invalid value for Packet".to_string(),
                }),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum BigPacket {
        Option1(u32),
        Option2(u32),
    }

    impl ToBytes for BigPacket {
        fn to_bytes(&self, buf: &mut [u8]) -> Result<usize, InsufficientBuffer> {
            if buf.len() < 5 {
                return Err(InsufficientBuffer);
            }

            let v = match self {
                Self::Option1(v) => {
                    buf[0] = 128;

                    v
                }
                Self::Option2(v) => {
                    buf[0] = 64;

                    v
                }
            };

            buf[1..5].copy_from_slice(&v.to_be_bytes());

            Ok(5)
        }
    }

    impl FromBytes for BigPacket {
        fn from_bytes(buf: &[u8]) -> Result<Self, FromByteError> {
            if buf.len() != 5 {
                return Err(FromByteError::InvalidData {
                    offset: 0,
                    desc: "Buffer has the wrong length".to_string(),
                });
            }

            let mut v_bytes = [0u8; 4];
            v_bytes.copy_from_slice(&buf[1..5]);
            let v = u32::from_be_bytes(v_bytes);

            match buf[0] {
                128 => Ok(Self::Option1(v)),
                64 => Ok(Self::Option2(v)),
                _ => Err(FromByteError::InvalidData {
                    offset: 0,
                    desc: "Invalid enum".to_string(),
                }),
            }
        }
    }

    #[test]
    fn test_inner_connect() {
        let (inner1, addr1, inner2, addr2) = get_inners();
        let mut buf = [0u8; 2];

        inner1.connect(addr2).unwrap();
        inner2.connect(addr1).unwrap();

        let first_packet = Packet::Option1;
        inner1.send(&first_packet, &mut buf).unwrap();

        let first_peeked_packet = inner2.peek(&mut buf).unwrap();
        assert_eq!(first_peeked_packet, first_packet);
        let first_recv_packet = inner2.recv(&mut buf).unwrap();
        assert_eq!(first_recv_packet, first_packet);
        assert_eq!(first_peeked_packet, first_recv_packet);

        let second_packet = Packet::Option2;
        inner2.send(&second_packet, &mut buf).unwrap();

        let second_peeked_packet = inner1.peek(&mut buf).unwrap();
        assert_eq!(second_peeked_packet, second_packet);
        let second_recv_packet = inner1.recv(&mut buf).unwrap();
        assert_eq!(second_recv_packet, second_packet);
        assert_eq!(second_peeked_packet, second_recv_packet);
    }

    #[test]
    fn test_inner_unconnected() {
        let (inner1, addr1, inner2, addr2) = get_inners();
        let mut buf = [0u8; 2];

        let first_packet = Packet::Option1;
        inner1.send_to(&first_packet, addr2, &mut buf).unwrap();

        let (first_peeked_packet, peeked_addr) = inner2.peek_from(&mut buf).unwrap();
        assert_eq!(peeked_addr, addr1);
        assert_eq!(first_peeked_packet, first_packet);
        let (first_recv_packet, addr) = inner2.recv_from(&mut buf).unwrap();
        assert_eq!(addr, addr1);
        assert_eq!(first_recv_packet, first_packet);
        assert_eq!(peeked_addr, addr);
        assert_eq!(first_peeked_packet, first_recv_packet);

        let second_packet = Packet::Option2;
        inner2.send_to(&second_packet, addr1, &mut buf).unwrap();

        let (second_peeked_packet, peeked_addr) = inner1.peek_from(&mut buf).unwrap();
        assert_eq!(peeked_addr, addr2);
        assert_eq!(second_peeked_packet, second_packet);
        let (second_recv_packet, addr) = inner1.recv_from(&mut buf).unwrap();
        assert_eq!(addr, addr2);
        assert_eq!(second_recv_packet, second_packet);
        assert_eq!(peeked_addr, addr);
        assert_eq!(second_peeked_packet, second_recv_packet);
    }

    #[test]
    fn test_inner_truncated() {
        use core::assert_matches;

        let (inner1, addr1, inner2, addr2) = get_inners();

        inner1.connect(addr2).unwrap();
        inner2.connect(addr1).unwrap();

        let mut big_buf = [0u8; 256];
        let first_packet = BigPacket::Option1(100);
        inner1.send(&first_packet, &mut big_buf).unwrap(); // Sufficient large buffer to fit `BigPacket`.

        let mut slightly_tiny_buf = [0u8; 4]; // Slightly too tiny buffer to fit a `BigPacket`.
        let mut fitting_buf = [0u8; 6]; // Exactly fitting buf with +1 space to detect truncation.
        assert_matches!(
            inner2.peek(&mut slightly_tiny_buf),
            Err(PeekError::DatagramTruncated)
        );
        let packet = inner2.recv(&mut fitting_buf).unwrap();
        assert_eq!(packet, first_packet);

        let second_packet = BigPacket::Option2(500);
        inner2.send(&second_packet, &mut big_buf).unwrap();

        let packet = inner1.peek(&mut fitting_buf).unwrap();
        assert_eq!(packet, second_packet);
        assert_matches!(
            inner1.recv(&mut slightly_tiny_buf),
            Err(RecvError::DatagramTruncated)
        );
    }

    fn get_inners<P>() -> (Inner<P>, SocketAddr, Inner<P>, SocketAddr)
    where
        P: Bytes,
    {
        let inner1: Inner<P> = Inner::bind("127.0.0.1:0").unwrap();
        let addr1 = inner1.local_addr().unwrap();
        let inner2: Inner<P> = Inner::bind("127.0.0.1:0").unwrap();
        let addr2 = inner2.local_addr().unwrap();

        (inner1, addr1, inner2, addr2)
    }
}
