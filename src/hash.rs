use std::fmt::Display;

use sha2::Digest;

use crate::traits::EndianessConversion;

#[derive(Debug)]
pub struct Sha256;

impl Sha256 {
    pub fn digest(data: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(data).to_vec()
    }

    /// Create a checksum with big endianess.
    pub fn checksum(data: &[u8]) -> [u8; 4] {
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&Self::digest(data)[..4]);
        checksum.to_be_bytes();

        checksum
    }

    pub fn verify_checksum(data: &[u8], checksum: &[u8]) -> bool {
        Self::checksum(data) == checksum
    }
}

#[derive(Debug)]
pub struct ChecksumMismatch;

impl Display for ChecksumMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Checksum Mismatch")
    }
}
