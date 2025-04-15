use core::ops::RangeInclusive;

/// Linear interpolation.
#[inline(always)]
pub(crate) fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + t * (end - start)
}

// Wraps a value into a range from 0 to modulus, correctly handling negative numbers.
#[inline(always)]
pub(crate) fn wrap(value: i32, modulus: i32) -> i32 {
    ((value % modulus) + modulus) % modulus
}

/// Round a floating point number to the nearest integer value as an f32
#[inline(always)]
pub(crate) fn round(x: f32) -> f32 {
    let integer_part = x as i32;
    let fractional_part = x - integer_part as f32;

    if fractional_part >= 0.5 {
        (integer_part + 1) as f32
    } else if fractional_part <= -0.5 {
        (integer_part - 1) as f32
    } else {
        integer_part as f32
    }
}

/// Floor a floating point number to the largest integer less than or equal to x as an f32
#[inline(always)]
pub(crate) fn floor(x: f32) -> f32 {
    let integer_part = x as i32;
    let fractional_part = x - integer_part as f32;

    if fractional_part < 0.0 {
        (integer_part - 1) as f32
    } else {
        integer_part as f32
    }
}

/// Truncate a floating point number to the integer component as an f32
#[inline(always)]
pub(crate) fn trunc(x: f32) -> f32 {
    let integer_part = x as i32;
    integer_part as f32
}

/// Maps a continuous value to one of a finite set of discrete values (steps)
/// that are evenly distributed within the specified range.
pub(crate) fn quantize_range(value: f32, steps: u16, range: RangeInclusive<f32>) -> f32 {
    // Zero steps returns zero, useful in setting the pan
    if steps == 0 {
        return 0.0;
    }
    // One step returns one, useful in channels without volume control, i.e. NES Triangle.
    if steps == 1 {
        return 1.0;
    }
    // Everything else...
    let steps = steps - 1;
    let min = *range.start();
    let max = *range.end();
    let step_size = (max - min) / steps as f32;
    // Find the nearest step by dividing the clamped value by step size, rounding it, and multiplying back
    let quantized_value = (round((value - min) / step_size) * step_size) + min;
    // Ensure the result is within the range after quantization
    quantized_value.clamp(min, max)
}
