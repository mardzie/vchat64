use crate::traits::InPlaceEndiannessConversion;

impl InPlaceEndiannessConversion for Vec<u8> {
    fn to_be(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_be();
        }
    }

    fn to_le(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_le();
        }
    }
}

impl InPlaceEndiannessConversion for [u8; 4] {
    fn to_be(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_be();
        }
    }

    fn to_le(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_le();
        }
    }
}
