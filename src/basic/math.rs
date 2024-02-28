use core::f32::consts::PI;
use libm::roundf;

pub const DEG_TO_RAD:f32 = PI / 180.0;
pub const RAD_TO_DEG:f32 = 180.0 / PI;

pub fn lerp(a:f32, b:f32, t:f32) -> f32 {
    a + ((b-a) * t)
}


pub fn quantize(value: f32, size: f32) -> f32 {
    roundf(value/size) * size
}

pub fn mirror_angle(angle: f32, mirror_normal: f32) -> f32 {
    2.0 * mirror_normal - angle
}


pub fn invert_angle(angle: f32) -> f32 {
    let inverted_angle = angle + core::f32::consts::PI;
    if inverted_angle > core::f32::consts::PI {
        inverted_angle - 2.0 * core::f32::consts::PI
    } else {
        inverted_angle
    }
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