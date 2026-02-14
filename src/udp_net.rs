//! This module handles all the in- and outgoing UDP traffic.

use std::{
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
    },
    thread,
};

use crate::helpers::calculate_version;

pub mod error;
pub mod packet;

use packet::{HEADER_LEN, Header, Packet};

/// The max payload size is 512 bytes.
///
/// This is to maximize througput and minimize latency and bytes lost.
pub const MAX_PAYLOAD_SIZE: usize = 512;
pub const MAX_PACKAGE_AGE_SEC: i64 = 10;

/// The UDP Socket handler.
///
/// There must never exists two identical `SocketAddr` in `addresses`!
#[derive(Debug)]
pub struct UdpNet {
    // tx_send: Sender<Packet>,
    // rx_read: Receiver<(SocketAddr, Vec<u8>)>,
    pub addresses: Arc<RwLock<Vec<SocketAddr>>>,
    writer_handle: thread::JoinHandle<()>,
    reader_handle: thread::JoinHandle<()>,
}

impl UdpNet {
    pub fn new<A>(
        addr: A,
        exit: Arc<AtomicBool>,
    ) -> Result<(Self, Sender<Packet>, Receiver<(SocketAddr, Vec<u8>)>), error::Error>
    where
        A: ToSocketAddrs,
    {
        let addresses = Arc::new(RwLock::new(Vec::with_capacity(8)));
        let writer_addresses = addresses.clone();
        let reader_addresses = addresses.clone();

        let socket_writer = UdpSocket::bind(addr)?;
        let socket_reader = socket_writer.try_clone()?;
        socket_writer
            .set_nonblocking(true)
            .expect("Failed to put socket into blocking mode!");

        let (tx_write, rx_write) = mpsc::channel::<Packet>();
        let (tx_read, rx_read) = mpsc::channel::<(SocketAddr, Vec<u8>)>();

        let exit_c = exit.clone();
        let writer_handle =
            thread::spawn(move || Self::writer(socket_writer, writer_addresses, rx_write, exit_c));
        let reader_handle =
            thread::spawn(move || Self::reader(socket_reader, tx_read, reader_addresses, exit));

        Ok((
            UdpNet {
                addresses,
                writer_handle,
                reader_handle,
            },
            tx_write,
            rx_read,
        ))
    }

    /// Writes packets from the write `Vec` to the stream.
    fn writer(
        socket: UdpSocket,
        addresses: Arc<RwLock<Vec<SocketAddr>>>,
        rx_write: Receiver<Packet>,
        exit: Arc<AtomicBool>,
    ) {
        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            let packet_bytes = match rx_write.try_recv() {
                Ok(mut packet) => {
                    packet.update_timestamp();
                    packet.into_bytes()
                }
                Err(e) => {
                    if let TryRecvError::Empty = e {
                        thread::yield_now();
                        continue;
                    } else {
                        log::warn!("Write channel closed: Stopping writer worker.");
                        break;
                    };
                }
            };

            for addr in addresses
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .iter()
            {
                match socket.send_to(&packet_bytes, addr) {
                    Ok(bytes_sent) => log::trace!("Wrote {} bytes to {}", bytes_sent, addr),
                    Err(e) => log::error!("Failed to write packet to {}: {}", addr, e),
                };
            }
        }

        log::info!("UDP Writer: Stopped.");
    }

    /// Reads from stream and pushed the packet onto the read `Vec`.
    fn reader(
        socket: UdpSocket,
        tx_read: Sender<(SocketAddr, Vec<u8>)>,
        addresses: Arc<RwLock<Vec<SocketAddr>>>,
        exit: Arc<AtomicBool>,
    ) {
        let packet_version: u32 = calculate_version();

        let mut buf = [0u8; u16::MAX as usize];
        let mut last_packet_timestamp = chrono::DateTime::UNIX_EPOCH;
        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            let (len, src_addr) = match socket.recv_from(&mut buf) {
                Ok((len, src_addr)) => (len, src_addr),
                Err(e) => {
                    if let std::io::ErrorKind::WouldBlock = e.kind() {
                        thread::yield_now();
                        continue;
                    } else {
                        log::error!("Failed to receive message: {}", e);
                        break;
                    };
                }
            };

            log::trace!("Read {} bytes from {}", len, src_addr);

            // Check if address is known.
            if !Self::contains_address(&addresses, &src_addr) {
                log::warn!(
                    "Received message from unknown origin: {}. Dropping packet.",
                    src_addr
                );
                continue;
            };

            if len < HEADER_LEN {
                log::warn!(
                    "Received too short message: {}, Minimum is {} bytes. Dropping packet.",
                    len,
                    HEADER_LEN
                );
                continue;
            };

            // Header
            let mut header_bytes = [0u8; HEADER_LEN];
            header_bytes.copy_from_slice(&buf[..HEADER_LEN]);
            let header = Header::from(header_bytes);

            // Header and Payload to bytes and checksum verification.
            let payload_bytes = buf[HEADER_LEN..len].to_vec();
            let packet = match Packet::new(header, payload_bytes) {
                Ok(packet) => packet,
                Err(e) => {
                    log::warn!("Corrupted packet: {}. Dropping packet.", e);
                    continue;
                }
            };

            // Version
            if packet.header().version() != packet_version {
                log::warn!("Version mismatch: Dropping packet.");
                continue;
            };

            // Timestamp
            let now = chrono::Utc::now();
            let max_age_timestamp =
                match now.checked_sub_signed(chrono::TimeDelta::seconds(MAX_PACKAGE_AGE_SEC)) {
                    Some(max_age_timestamp) => max_age_timestamp,
                    None => {
                        log::error!(
                            "Failed to subtract {} s from {}: Dropping packet.",
                            MAX_PACKAGE_AGE_SEC,
                            now
                        );
                        continue;
                    }
                };
            let packet_timestamp = packet.header().timestamp();
            if packet_timestamp > now {
                log::warn!("Invalid packet timestamp: Packet is from the future. Dropping packet.");
                continue;
            } else if packet_timestamp < max_age_timestamp {
                log::warn!("Invalid packet timestamp: Packet is too old. Dropping packet.");
                continue;
            } else if packet_timestamp < last_packet_timestamp {
                log::warn!(
                    "Invalid packet timestamp: Packet is older than the most recent packet. Dropping packet."
                );
                continue;
            };
            last_packet_timestamp = packet_timestamp;

            let (_, payload) = packet.split();
            match tx_read.send((src_addr, payload)) {
                Ok(_) => {}
                Err(_) => {
                    log::error!(
                        "Received packet receiver has shut down: Shutting down UDP receiver and dropping received packets."
                    );
                    continue;
                }
            };

            log::debug!("Received valid message.");
        }

        log::info!("UDP Reader: Stopped.");
    }

    #[inline]
    pub fn is_address_known(&self, addr: &SocketAddr) -> bool {
        Self::contains_address(&self.addresses, addr)
    }

    #[inline]
    fn contains_address(addresses: &Arc<RwLock<Vec<SocketAddr>>>, addr: &SocketAddr) -> bool {
        addresses
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains(addr)
    }

    pub fn stop(self) {
        let _ = self.reader_handle.join();
        let _ = self.writer_handle.join();
    }

    // #[inline]
    // pub fn clone_sender_channel(&self) -> Sender<Packet> {
    //     self.tx_send.clone()
    // }

    // #[inline]
    // pub fn ref_recv_channel(&self) -> &Receiver<(SocketAddr, Vec<u8>)> {
    //     &self.rx_read
    // }

    // /// Send a voice packet.
    // ///
    // /// When sucessful `Ok(())` is returned.
    // /// On error the bytes will be returned.
    // /// When an error is returned this function will never work again.
    // #[inline]
    // pub fn send(&self, bytes: Vec<u8>) -> Result<(), Vec<u8>> {
    //     match self.tx_send.send(Packet::from(bytes)) {
    //         Ok(_) => Ok(()),
    //         Err(e) => Err(e.0.payload),
    //     }
    // }

    // /// Receive a voice packet.
    // #[inline]
    // pub fn recv(&self) -> Result<Option<(SocketAddr, Vec<u8>)>, error::Error> {
    //     match self.rx_read.try_recv() {
    //         Ok(packet) => Ok(Some(packet)),
    //         Err(e) => match e {
    //             TryRecvError::Empty => Ok(None),
    //             TryRecvError::Disconnected => Err(error::Error::SocketClosed("Reader closed")),
    //         },
    //     }
    // }
}
