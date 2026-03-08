use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};

pub mod error;
pub mod packet;

use packet::{HEADER_LEN, Header, Packet};

use crate::TIMEOUT;

pub const MAX_PACKET_SIZE: usize = 512;
/// The max payload size is 512 bytes.
///
/// This is to maximize throughput and minimize latency and bytes lost.
pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_LEN;

/// The UDP Socket handler.
///
/// There must never exists two identical `SocketAddr` in `addresses`!
#[derive(Debug)]
pub struct UdpPacketNet {
    socket: UdpSocket,

    recv_buf: [u8; u16::MAX as usize],
}

impl UdpPacketNet {
    pub fn new<A>(addr: A) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let socket = UdpSocket::bind(addr).map_err(|e| error::Error::SocketBind(e))?;
        socket
            .set_nonblocking(true)
            .expect("Failed to put socket into blocking mode!");
        socket
            .set_read_timeout(Some(TIMEOUT))
            .expect("Failed to set UDP Socket read timeout.");
        socket
            .set_write_timeout(Some(TIMEOUT))
            .expect("Failed to set UDP Socket write timeout.");

        Ok(UdpPacketNet {
            socket,
            recv_buf: [0u8; u16::MAX as usize],
        })
    }

    /// Sends the `packet` to the given address.
    ///
    /// This operation is non blocking.
    ///
    /// # Error:
    ///
    /// On blocking behavior `Error::WouldBlock` is returned.
    pub fn send<A>(&self, packet: Packet, addr: A) -> Result<usize, error::Error>
    where
        A: ToSocketAddrs,
    {
        self.socket
            .send_to(&packet.into_bytes(), addr)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    error::Error::WouldBlock
                } else {
                    error::Error::Send(e)
                }
            })
    }

    /// Reads a [`Packet`] from stream and returns the `Packet` and the source `SocketAddr`.
    ///
    /// This operation in non blocking.
    ///
    /// # Error:
    ///
    /// On blocking behavior `io::ErrorKind::WouldBlock` is returned.
    pub fn recv(&mut self) -> Result<(Packet, SocketAddr), error::Error> {
        let (len, addr) = self
            .socket
            .recv_from(&mut self.recv_buf)
            .map_err(|e| error::Error::Recv(e))?;

        // Header
        let mut header_bytes = [0u8; HEADER_LEN];
        header_bytes.copy_from_slice(&self.recv_buf[..HEADER_LEN]);
        let header = Header::from(header_bytes);

        // Header and Payload to bytes and checksum verification.
        let payload_bytes = self.recv_buf[HEADER_LEN..len].to_vec();
        let packet = match Packet::new(header, payload_bytes) {
            Ok(packet) => packet,
            Err(_) => return Err(error::Error::ChecksumMismatch),
        };

        Ok((packet, addr))
    }
}
