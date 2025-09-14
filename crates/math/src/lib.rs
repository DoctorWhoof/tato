#![no_std]

pub mod num;
pub use num::{Float, Integer, Num, SignedNum};

pub mod prelude {
    pub use crate::num::{Float, Integer, Num, SignedNum};
    pub use crate::rect::*;
    pub use crate::vec2::*;
}

pub use libm;

pub mod rect;
pub use rect::Rect;

pub mod vec2;
pub use vec2::Vec2;

// Skips quantization if value is too tiny, useful when getting elapsed time in
// immediate timing mode and very fast frame rates.
#[inline]
pub fn quantize(value: f32, size: f32) -> f32 {
    let result = (value / size).floor() * size;
    if result < f32::EPSILON { value } else { result }
}

pub fn next_power_of_two(mut n: u32) -> u32 {
    if n == 0 {
        return 1;
    }
    if n.is_power_of_two() {
        return n;
    }
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;

    if cfg!(debug_assertions) {
        // Check for overflow before adding 1
        if n == u32::MAX { u32::MAX } else { n + 1 }
    } else {
        n + 1
    }
}

pub fn prev_power_of_two(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    }
    if n.is_power_of_two() {
        return n;
    }
    1 << (31 - n.leading_zeros())
}
