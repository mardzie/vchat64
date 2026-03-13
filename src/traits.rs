pub mod num;

pub trait InPlaceEndiannessConversion {
    fn to_be(&mut self);

    fn to_le(&mut self);
}
