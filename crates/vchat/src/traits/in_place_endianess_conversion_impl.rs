use crate::traits::InPlaceEndiannessConversion;

impl InPlaceEndiannessConversion for [u8; 4] {
    fn to_be(&mut self) {
        *self = u32::from_le_bytes(*self).to_be_bytes();
    }

    fn to_le(&mut self) {
        *self = u32::from_be_bytes(*self).to_be_bytes();
    }
}
