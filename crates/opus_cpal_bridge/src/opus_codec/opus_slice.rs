#[derive(Debug)]
pub struct OpusSlice<'a>(&'a [u8]);

impl<'a> OpusSlice<'a> {
    pub fn new(s: &'a [u8], sample_rate: u32) -> Result<Self, Error> {
        if !s.is_empty() {
            if s.len() % 120 == 0 {
                Ok(Self(s))
            } else {
                Err(Error::InvalidSize)
            }
        } else {
            Err(Error::Empty)
        }
    }

    pub fn get(&self) -> &'a [u8] {
        self.0
    }
}

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Error {
    #[error("empty slice")]
    Empty,
    #[error("not a multiple of 120")]
    InvalidSize,
}
