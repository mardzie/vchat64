use std::{
    collections::{self, BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    net::ToSocketAddrs,
    sync::{Arc, atomic::AtomicBool},
};

use crate::{
    helpers::calculate_version,
    udp_packet_net::{self, UdpPacketNet},
};

mod error;

#[derive(Debug)]
pub struct VoiceNet {
    packet_net: UdpPacketNet,
    exit: Arc<AtomicBool>,

    incoming_packet_buf: VecDeque<Vec<u8>>,
}

impl VoiceNet {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let packet_net = UdpPacketNet::new(addr)?;

        Ok(Self {
            packet_net,
            exit,
            incoming_packet_buf: VecDeque::with_capacity(1024),
        })
    }

    pub fn send(&self) {}

    pub fn recv(&mut self) {
        let version = calculate_version();

        let (packet, src_addr) = match self.packet_net.recv() {
            Ok(packet) => packet,
            Err(e) => match e {
                udp_packet_net::error::Error::Recv(e) => {
                    log::warn!("Failed to receive packet from UDP Packet Net: {}", e);
                    return;
                }
                udp_packet_net::error::Error::ChecksumMismatch => {
                    log::warn!("UDP Packet Checksum Mismatch.");
                    return;
                }
                _ => {
                    panic!(
                        "Invalid Error Variant returned! in `voice_net.rs` in `VoiceNet::recv()`"
                    );
                }
            },
        };

        // Version
        if packet.header().version() != version {
            log::warn!("Version mismatch: Dropping packet.");
            return;
        };

        // Timestamp
        todo!();
    }
}
