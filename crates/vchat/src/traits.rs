pub mod in_place_endianess_conversion_impl;

pub trait InPlaceEndiannessConversion {
    fn to_be(&mut self);

    fn to_le(&mut self);
}
