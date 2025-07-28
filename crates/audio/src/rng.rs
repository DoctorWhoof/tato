//! A simple, "old school" LFSR with configurable bit count.
#[derive(Debug)]
pub struct Rng {
    state: u32,
    mask: u32,
    tap: u32,
    f32_max: f32,
}

const DEFAULT_VALUE: u32 = 0b_01100010_10101111_01101010_10101011;

impl Rng {
    /// Creates a new LFSR with `n` bits and an initial state.
    pub fn new(bit_count: u32, seed: u32) -> Self {
        debug_assert!(bit_count >= 3 && bit_count <= 32, "bit_count must be between 3 and 32");
        let bit_count = bit_count.clamp(3, 32);
        let mask = ((1u64 << bit_count) - 1) as u32;
        let state = if (seed & mask) == 0 {
            // print!("Overriding Inital State: ");
            DEFAULT_VALUE & mask
        } else {
            // print!("Inital State: ");
            seed & mask
        };
        // println!(" 0x{:X}", state);
        Self {
            state,
            mask,
            tap: tap(bit_count),
            f32_max: (1u64 << bit_count) as f32, // works with bit count up to 63, so we're good
        }
    }

    #[allow(unused)]
    #[inline]
    pub fn max_value(&self) -> u32 {
        self.mask
    }

    /// Next random u32 value in the sequence (using Galois LFSR with scrambling)
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        // Galois LFSR implementation
        let output = self.state;
        let feedback = (self.state & 1).wrapping_neg(); // 0 -> 0x00000000, 1 -> 0xFFFFFFFF
        self.state = (self.state >> 1) ^ (self.tap & feedback);

        // Apply scrambling to break correlations
        scramble(output & self.mask)
    }

    /// Converts next random u32 value to range (0.0 .. 1.0)
    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / self.f32_max
    }

    /// Generate a random u32 in the inclusive range [min, max]
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min <= max, "min must be <= max");
        if min == max {
            return min;
        }

        let range = max - min + 1;

        // Handle power-of-2 ranges efficiently
        if range.is_power_of_two() {
            return min + (self.next_u32() & (range - 1));
        }

        // Use rejection sampling to avoid modulo bias
        let mask = (!0u32) >> range.leading_zeros();
        loop {
            let candidate = self.next_u32() & mask;
            if candidate < range {
                return min + candidate;
            }
        }
    }

    /// Generate a random i32 in the inclusive range [min, max]
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        debug_assert!(min <= max, "min must be <= max");
        if min == max {
            return min;
        }

        // Convert to u32 range to avoid overflow issues
        let range_size = (max as i64 - min as i64 + 1) as u32;
        min + self.range_u32(0, range_size - 1) as i32
    }

    /// Generate a random f32 in the range [min, max)
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        debug_assert!(min <= max, "min must be <= max");
        min + self.next_f32() * (max - min)
    }
}

#[inline]
fn scramble(mut x: u32) -> u32 {
    // Avalanche function to break correlations between consecutive outputs
    x ^= x >> 16;
    x = x.wrapping_mul(0x85ebca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2ae35);
    x ^= x >> 16;
    x
}

fn tap(bit_count: u32) -> u32 {
    match bit_count {
        3 => 0b110,
        4 => 0b1100,
        5 => 0b10100,
        6 => 0b110000,
        7 => 0b1100000,
        8 => 0b10111000,
        9 => 0b100010000,
        10 => 0b1001000000,
        11 => 0b10100000000,
        12 => 0b111000001000,
        13 => 0b1110010000000,
        14 => 0b11100000000010,
        15 => 0b110000000000000,
        16 => 0b1011010000000000,
        17 => 0b10010000000000000,
        18 => 0b100000010000000000,
        19 => 0b1110010000000000000,
        20 => 0b10010000000000000000,
        21 => 0b101000000000000000000,
        22 => 0b1100000000000000000000,
        23 => 0b10000100000000000000000,
        24 => 0b100000010000000000000000,
        25 => 0b1000100000000000000000000,
        26 => 0b10010000000000000000000000,
        27 => 0b101000000000000000000000000,
        28 => 0b1100100000000000000000000000,
        29 => 0b11000000000000000000000000000,
        30 => 0b110010000000000000000000000000,
        31 => 0b1101000000000000000000000000000,
        32 => 0x80200003,
        _ => 0b101101,
    }
}
