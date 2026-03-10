use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign,
    Sub, SubAssign,
};

pub trait Num: Add + Sub + Mul + Div + Rem + Sized {}
pub trait NumAssign: AddAssign + SubAssign + MulAssign + DivAssign + RemAssign + Sized {}

pub trait NumCmp: Eq + Ord + Sized {}
pub trait NumPartialCmp: PartialEq + PartialOrd + Sized {}

pub trait NumSh: Shl + Shr + Sized {}
pub trait NumShAssign: ShlAssign + ShrAssign + Sized {}
