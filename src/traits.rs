pub mod num;

pub trait InPlaceEndiannessConversion {
    fn to_be(&mut self);

    fn to_le(&mut self);
}

pub trait CopyFromIterator<T> {
    fn copy_from_iter<I: IntoIterator<Item = T>>(&mut self, iter: I);
}
