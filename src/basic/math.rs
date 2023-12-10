

pub fn quantize(value: f32, size: f32) -> f32 {
    (value/size).round() * size
}


pub trait MinMax {
    fn min(self, other:Self) -> Self;
    fn max(self, other:Self) -> Self;
}

impl MinMax for f32 {
    fn min(self, other:Self) -> Self {self.min(other)}
    fn max(self, other:Self) -> Self {self.max(other)}
}

impl MinMax for i32 {
    fn min(self, other:Self) -> Self {core::cmp::Ord::min(self, other)}
    fn max(self, other:Self) -> Self {core::cmp::Ord::max(self, other)}
}

impl MinMax for i8 {
    fn min(self, other:Self) -> Self {core::cmp::Ord::min(self, other)}
    fn max(self, other:Self) -> Self {core::cmp::Ord::max(self, other)}
}


// use std::ops::{Add, Div, Mul};

// pub trait Roundable { fn round(self) -> Self; }

// impl Roundable for f32 {
//     fn round(self) -> Self {self.round()}
// }

// impl Roundable for f64 {fn round(self) -> Self {self.round()}
// }

// pub fn quantize<T>(value: T, size: T) -> T
// where T: Copy + Add<Output = T> + Div<Output = T> + Mul<Output = T> + Roundable {
//     (value/size).round() * size
// }