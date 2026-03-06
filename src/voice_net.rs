use std::{
    net::ToSocketAddrs,
    sync::{Arc, atomic::AtomicBool},
};

use crate::udp_net::UdpPacketNet;

mod error;

#[derive(Debug)]
pub struct VoiceNet {
    packet_net: UdpPacketNet,
    exit: Arc<AtomicBool>,
}

impl VoiceNet {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        let packet_net = UdpPacketNet::new(addr)?;
        Ok(Self { packet_net, exit })
    }

    pub fn write(&self) {}

    pub fn read(&self) {
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
            // TODO: Rejects too many packets if just a bit in the future.
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

        log::trace!("UDP Reader: Received valid message.");
    }
}
