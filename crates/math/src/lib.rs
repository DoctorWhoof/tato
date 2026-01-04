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

/// Linear interpolation.
/// Pre-multiply "t" by delta time if needed.
#[inline(always)]
pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (t * (end - start))
}

/// Smooth interpolation using smoothstep function.
/// Pre-multiply "t" by delta time if needed.
#[inline(always)]
pub fn smerp(start: f32, end: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let smooth_t = t * t * (3.0 - 2.0 * t);
    start + smooth_t * (end - start)
}

/// Maps a continuous value to a step size.
#[inline]
pub fn quantize(value: f32, size: f32) -> f32 {
    (value / size).floor() * size
    // Skip quantization if value is too tiny.
    // if result < f32::EPSILON { value } else { result }
}

/// Wraps a value into a range from 0 to modulus, correctly handling negative numbers.
#[inline]
pub fn wrap(value: i32, modulus: i32) -> i32 {
    ((value % modulus) + modulus) % modulus
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
