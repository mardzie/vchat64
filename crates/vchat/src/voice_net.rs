use std::{
    collections::VecDeque,
    io,
    net::{SocketAddr, ToSocketAddrs},
};

use crate::{
    helpers::calculate_version,
    udp_packet_net::{self, UdpPacketNet, packet::Packet},
};

pub mod error;

const PACKET_BUF_TIME_IN_QUEUE: chrono::Duration = chrono::Duration::milliseconds(100);

pub type PacketTuple = (SocketAddr, Vec<u8>);

#[derive(Debug)]
pub struct VoiceNet {
    current_packet_version: u32,

    packet_net: UdpPacketNet,

    incoming_packet_buf: VecDeque<(chrono::DateTime<chrono::Utc>, PacketTuple)>,
}

impl VoiceNet {
    pub fn new<A>(addr: A) -> Result<Self, io::Error>
    where
        A: ToSocketAddrs,
    {
        let packet_net = UdpPacketNet::new(addr)?;
        let current_packet_version = calculate_version();

        Ok(Self {
            current_packet_version,
            packet_net,
            incoming_packet_buf: VecDeque::with_capacity(1024),
        })
    }

    /// Send a packet.
    ///
    /// This function does not block.
    pub fn send<A>(&self, data: Vec<u8>, addr: &A) -> Result<(), error::SendError>
    where
        A: ToSocketAddrs,
    {
        match self.packet_net.send(Packet::from(data), addr) {
            Ok(_) => Ok(()),
            Err(e) => Err(error::SendError::from(e)),
        }
    }

    /// Tries to receives a packet from queue. If no packet is available `None` is returned.
    ///
    /// This function does not block.
    pub fn recv(&mut self) -> Option<(chrono::DateTime<chrono::Utc>, PacketTuple)> {
        match self.read_packet() {
            Ok(_) => {}
            Err(e) => {
                log::warn!("Failed to read packet: {}", e);
            }
        }

        // Keep `PACKET_BUF_TIME_IN_QUEUE` packets in the queue. This helps to keep as many packets as possible.
        let (dt, packet) = self.incoming_packet_buf.pop_front()?;
        let now = chrono::Utc::now();
        if now - dt > PACKET_BUF_TIME_IN_QUEUE {
            Some((dt, packet))
        } else {
            self.incoming_packet_buf.push_front((dt, packet));
            None
        }
    }

    /// Tries to read a new packet.
    ///
    /// # Error
    ///
    /// Returns `Err(())` if no valid packet could be read.
    fn read_packet(&mut self) -> Result<(), String> {
        let (packet, src_addr) = match self.packet_net.recv() {
            Ok(packet) => packet,
            Err(e) => match e {
                udp_packet_net::error::RecvError::Io(e) => {
                    return Err(format!(
                        "Failed to receive packet from UDP Packet Net: {}",
                        e
                    ));
                }
                udp_packet_net::error::RecvError::ChecksumMismatch => {
                    return Err("UDP Packet Checksum Mismatch.".to_string());
                }
                udp_packet_net::error::RecvError::WouldBlock => return Ok(()),
            },
        };

        // Version
        if packet.header().version() != self.current_packet_version {
            return Err("Version mismatch: Dropping packet.".to_string());
        };

        let packet_timestamp = packet.header().timestamp();

        // Do not insert too old packets.
        let idx = self
            .incoming_packet_buf
            .partition_point(|(ts, _)| ts < &packet_timestamp);
        if idx > 0 || self.incoming_packet_buf.is_empty() {
            self.incoming_packet_buf
                .insert(idx, (packet_timestamp, (src_addr, packet.payload)));
            Ok(())
        } else {
            Err(format!(
                "Packet too old: Packet {} < {} Oldest",
                packet_timestamp,
                self.incoming_packet_buf.front().expect("Has to be Some").0 // Has to be Some or the If should have been chosen.
            ))
        }
    }
}
