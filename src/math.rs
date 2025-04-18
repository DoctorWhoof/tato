use core::ops::RangeInclusive;

use libm::roundf;

// /// Linear interpolation.
// #[inline(always)]
// pub(crate) fn lerp(start: f32, end: f32, t: f32) -> f32 {
//     start + t * (end - start)
// }

/// Returns the MIDI note value given an octave (zero to 10) and a note (zero to 11).
pub fn get_midi_note(octave: impl Into<i32>, note: impl Into<i32>) -> i32 {
    // Handle negative values and values beyond range
    let octave = wrap(octave.into(), 10);
    let note = wrap(note.into(), 12);
    // MIDI note number, where C4 is 60
    ((octave + 1) * 12) + note
}

#[inline(always)]
/// The frequency in Hz of any MIDI note value.
pub fn note_to_frequency(note: f32) -> f32 {
    libm::powf(2.0, (note - 69.0) / 12.0) * 440.0
}

#[inline(always)]
/// The corresponding note of a frequency
pub fn frequency_to_note(frequency: f32) -> f32 {
    69.0 + 12.0 * libm::log2f(frequency / 440.0)
}

// Wraps a value into a range from 0 to modulus, correctly handling negative numbers.
#[inline(always)]
pub(crate) fn wrap(value: i32, modulus: i32) -> i32 {
    ((value % modulus) + modulus) % modulus
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
    let quantized_value = (roundf((value - min) / step_size) * step_size) + min;
    // Ensure the result is within the range after quantization
    quantized_value.clamp(min, max)
}
