use crate::{
    calculate_version,
    hash::{self},
};

pub const HEADER_LEN: usize = 4 + 8 + 4;

#[derive(Debug, Clone, Hash)]
pub struct Packet {
    pub header: Header,
    pub payload: Vec<u8>,
}

/// Header
/// # Protocol
/// | VERSION | TIMESTAMP ms | CHECKSUM | PAYLOAD   |
/// | ------- | ------------ | -------- | --------- |
/// | 4 be    | 8 be in ms  | 4 be     | XXX be    |
///
/// Version is Major * 10^(Minor Places) + Minor.
/// e. g. Major 2; Minor 50;
/// 2 * 10^2 = 200
/// 200 + 50 = 250
/// Version is 250
#[derive(Debug, Clone, Hash)]
pub struct Header {
    version: u32,
    timestamp: chrono::DateTime<chrono::Utc>,
    checksum: [u8; 4],
}

impl Packet {
    pub fn new(header: Header, payload: Vec<u8>) -> Result<Self, hash::ChecksumMismatch> {
        if header.verify_checksum(&payload) {
            Ok(Self { header, payload })
        } else {
            Err(hash::ChecksumMismatch)
        }
    }

    pub fn verify_checksum(&self) -> bool {
        self.header.verify_checksum(&self.payload)
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn split(self) -> (Header, Vec<u8>) {
        (self.header, self.payload)
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let mut buf = self.payload;
        let len = buf.len();
        buf.reserve_exact(HEADER_LEN);
        buf.copy_within(..len, HEADER_LEN);

        self.header.to_bytes(&mut buf[..HEADER_LEN]);

        buf
    }
}

impl From<Vec<u8>> for Packet {
    fn from(payload: Vec<u8>) -> Self {
        let header = Header::new(&payload);
        Self { header, payload }
    }
}

impl Header {
    pub fn new(payload: &[u8]) -> Self {
        let len = payload.len();
        if len < u16::MAX as usize {
            panic!(
                "Payload too big: Payload must be less or equal to {}",
                u16::MAX
            );
        };

        let checksum = hash::Sha256::checksum(payload);

        Self {
            version: calculate_version(),
            timestamp: chrono::Utc::now(),
            checksum,
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    pub fn verify_checksum(&self, payload: &[u8]) -> bool {
        hash::Sha256::verify_checksum(payload, &self.checksum)
    }

    /// Converts [`Header`] into bytes and write it into `buf`.
    ///
    /// `buf` requires exactly [`HEADER_LEN`] space.
    pub fn to_bytes(&self, buf: &mut [u8]) {
        if buf.len() != HEADER_LEN {
            panic!(
                "`buf` requires exactly `HEADER_LEN` ({}) space.",
                HEADER_LEN
            );
        };

        buf[..4].copy_from_slice(&self.version.to_be_bytes());
        buf[4..12].copy_from_slice(&self.timestamp.timestamp_millis().to_be_bytes());
        buf[12..HEADER_LEN].copy_from_slice(&self.checksum);
    }

    pub fn from_bytes(header_bytes: [u8; HEADER_LEN]) -> Self {
        let mut version = [0u8; 4];
        let mut timestamp = [0u8; 8];
        let mut checksum = [0u8; 4];

        version.copy_from_slice(&header_bytes[..4]);
        timestamp.copy_from_slice(&header_bytes[4..12]);
        checksum.copy_from_slice(&header_bytes[12..HEADER_LEN]);

        let timestamp_number = i64::from_be_bytes(timestamp);
        let timestamp = match chrono::DateTime::from_timestamp_millis(timestamp_number) {
            Some(timestamp) => timestamp,
            None => {
                log::warn!(
                    "Failed to get `DateTime` from {}: Defaulting to UNIX epoch.",
                    timestamp_number
                );
                chrono::DateTime::UNIX_EPOCH
            }
        };

        Self {
            version: u32::from_be_bytes(version),
            timestamp,
            checksum,
        }
    }
}
