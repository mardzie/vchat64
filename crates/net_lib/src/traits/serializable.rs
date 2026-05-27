pub trait ToBytes {
    /// Encode `&self` into `&mut Vec<u8>`.
    fn to_bytes(&self, buf: &mut Vec<u8>);
}

pub trait FromBytes: Sized {
    /// Decode `&[u8]` into `Self`.
    fn from_bytes(buf: &[u8]) -> Result<Self, FromByteError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FromByteError {
    #[error("Unexpected end of file: {needed} bytes needed but only {available} bytes: {desc} ")]
    UnexpectedEOF {
        needed: usize,
        available: usize,
        desc: String,
    },
    #[error("Invalid data at {offset}: {desc}")]
    InvalidData { offset: usize, desc: String },
}

pub trait Bytes: ToBytes + FromBytes {}
impl<T: ToBytes + FromBytes> Bytes for T {}
