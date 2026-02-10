pub trait EndianessConversion {
    fn to_be_bytes(&mut self);

    fn to_le_bytes(&mut self);
}
