use std::{
    collections::VecDeque,
    net::{SocketAddr, ToSocketAddrs},
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use crate::{
    helpers::calculate_version,
    udp_packet_net::{self, UdpPacketNet},
};

mod error;

const PACKET_BUF_TIME_IN_QUEUE: chrono::Duration = chrono::Duration::milliseconds(100);

#[derive(Debug)]
pub struct VoiceNet {
    current_packet_version: u32,

    packet_net: UdpPacketNet,
    exit: Arc<AtomicBool>,

    incoming_packet_buf:
        Arc<Mutex<VecDeque<(chrono::DateTime<chrono::Utc>, (SocketAddr, Vec<u8>))>>>,
}

impl VoiceNet {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let packet_net = UdpPacketNet::new(addr)?;
        let current_packet_version = calculate_version();

        Ok(Self {
            current_packet_version,
            packet_net,
            exit,
            incoming_packet_buf: Arc::new(Mutex::new(VecDeque::with_capacity(1024))),
        })
    }

    pub fn send(&self) {}

    /// Tries to receives a packet from queue
    pub fn recv(&mut self) -> Option<(chrono::DateTime<chrono::Utc>, (SocketAddr, Vec<u8>))> {
        match self.read_packet() {
            Ok(_) => {}
            Err(_) => {
                log::warn!("Failed to read packet.");
            }
        }

        let mut packet_buf = self
            .incoming_packet_buf
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Keep `PACKET_BUF_TIME_IN_QUEUE` packets in the queue. This helps to keep as many packets as possible.
        let (dt, packet) = packet_buf.pop_front()?;
        let now = chrono::Utc::now();
        if now - dt > PACKET_BUF_TIME_IN_QUEUE {
            Some((dt, packet))
        } else {
            packet_buf.push_front((dt, packet));
            None
        }
    }

    /// Tries to read a new packet.
    ///
    /// # Error
    ///
    /// Returns `Err(())` if no valid packet could be read.
    fn read_packet(&mut self) -> Result<(), ()> {
        let (packet, src_addr) = match self.packet_net.recv() {
            Ok(packet) => packet,
            Err(e) => match e {
                udp_packet_net::error::Error::Recv(e) => {
                    log::warn!("Failed to receive packet from UDP Packet Net: {}", e);
                    return Err(());
                }
                udp_packet_net::error::Error::ChecksumMismatch => {
                    log::warn!("UDP Packet Checksum Mismatch.");
                    return Err(());
                }
                _ => {
                    panic!(
                        "Invalid Error Variant returned! in `voice_net.rs` in `VoiceNet::recv()`"
                    );
                }
            },
        };

        // Version
        if packet.header().version() != self.current_packet_version {
            log::warn!("Version mismatch: Dropping packet.");
            return Err(());
        };

        let packet_timestamp = packet.header().timestamp();

        let mut packet_buf = self
            .incoming_packet_buf
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Do not insert too old packets.
        let idx = packet_buf.partition_point(|(ts, _)| ts < &packet_timestamp);
        if idx > 0 {
            packet_buf.insert(idx, (packet_timestamp, (src_addr, packet.payload)));
            Ok(())
        } else {
            Err(())
        }
    }
}
