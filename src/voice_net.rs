use std::{
    net::{ToSocketAddrs, UdpSocket},
    sync::{Arc, atomic::AtomicBool},
};

mod error;

#[derive(Debug)]
pub struct VoiceNet {
    socket: UdpSocket,
}

impl VoiceNet {
    pub fn new<A>(addr: A, exit: Arc<AtomicBool>) -> Result<Self, error::Error>
    where
        A: ToSocketAddrs,
    {
        Ok(Self { udp_net })
    }

    pub fn write(&self) {}

    pub fn read(&self) {
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
