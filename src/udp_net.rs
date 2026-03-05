//! This module handles all the in- and outgoing UDP traffic.

use std::{
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, TryRecvError},
    },
    thread,
};

use crate::helpers::calculate_version;

pub mod error;
pub mod packet;

use packet::{HEADER_LEN, Header, Packet};

pub const MAX_PACKET_SIZE: usize = 512;
/// The max payload size is 512 bytes.
///
/// This is to maximize throughput and minimize latency and bytes lost.
pub const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - HEADER_LEN;
pub const MAX_PACKAGE_AGE_SEC: i64 = 10;

/// The UDP Socket handler.
///
/// There must never exists two identical `SocketAddr` in `addresses`!
#[derive(Debug)]
pub struct UdpNet {
    socket: UdpSocket,
}

impl UdpNet {
    pub fn new<A>(addr: A) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let socket = UdpSocket::bind(addr)?;
        socket
            .set_nonblocking(true)
            .expect("Failed to put socket into blocking mode!");

        Ok(UdpNet { socket })
    }

    /// Sends `bytes` to the given addr.
    fn send_to<A>(&self, bytes: &[u8], addr: A) -> Result<usize, io::Error>
    where
        A: ToSocketAddrs,
    {
        self.socket.send_to(bytes, addr)
    }

    /// Reads a packet from stream and stores it in `buf` and returns the number of bytes read and the source `SocketAddr`.
    ///
    /// Buf should have at least [`MAX_PACKET_SIZE`] bytes capacity.
    fn recv(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), io::Error> {
        self.socket.recv_from(buf)
    }
}
