use core::ops::{Add, BitXor, Rem, Shl, Shr, Sub};

// Trait for numeric types that can be used as RNG state
pub trait RngInteger:
    Copy
    + BitXor<Output = Self>
    + Shl<u8, Output = Self>
    + Shr<u8, Output = Self>
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Rem<Output = Self>
{
    // Get max value for this integer type
    fn max_value() -> Self;

    // Get a suitable non-zero seed value
    fn default_seed() -> Self;

    // Convert to usize for indexing operations
    fn to_usize(self) -> usize;

    // Convert from usize for range calculations
    fn from_usize(value: usize) -> Self;

    // Convert from u8 for simple constants
    fn from_u8(value: u8) -> Self;
}

pub struct SimpleRng<T: RngInteger> {
    state: T,
}

impl<T: RngInteger> SimpleRng<T> {
    pub fn new(seed: T) -> Self {
        Self {
            state: if seed == T::from_u8(0) {
                T::default_seed()
            } else {
                seed
            },
        }
    }

    // Generate the next random value
    pub fn next(&mut self) -> T {
        // Apply a simplified xorshift algorithm appropriate for any integer size
        self.state = self.state.bitxor(self.state.shl(3));
        self.state = self.state.bitxor(self.state.shr(2));
        self.state = self.state.bitxor(self.state.shl(1));
        self.state
    }

    // Generate a value in the inclusive range [start, end]
    pub fn gen_range<U: RngInteger>(&mut self, start: U, end: U) -> U {
        if start == end {
            return start;
        }

        // Prevents overflow since range is inclusive
        let end = if end == U::max_value() {
            end - U::from_u8(1)
        } else {
            end
        };

        let state_val = self.next().to_usize();
        let range_size = end - start + U::from_u8(1);
        let range_size_usize = range_size.to_usize();
        let random_offset = state_val % range_size_usize;

        start + U::from_usize(random_offset)
    }

    // Generate a normalized floating point value between 0.0 and 1.0
    #[allow(unused)]
    pub fn gen_float(&mut self) -> f32
    where
        T: Into<f32>,
    {
        let value: f32 = self.next().into();
        let max: f32 = T::max_value().into();
        (value / max)
    }
}

// Macro to implement RngInteger for integer types
macro_rules! impl_rng_integer {
    ($type:ty, $default_seed:expr) => {
        impl RngInteger for $type {
            #[inline]
            fn max_value() -> Self {
                <$type>::MAX
            }

            #[inline]
            fn default_seed() -> Self {
                $default_seed
            }

            #[inline]
            fn to_usize(self) -> usize {
                self as usize
            }

            #[inline]
            fn from_usize(value: usize) -> Self {
                value as $type
            }

            #[inline]
            fn from_u8(value: u8) -> Self {
                value as $type
            }
        }
    };
}

// Implement for common unsigned integer types
impl_rng_integer!(u8, 0xAB);
impl_rng_integer!(u16, 0xABCD);
// impl_rng_integer!(u32, 0xABCD_EF01);
// impl_rng_integer!(u64, 0xABCD_EF01_2345_6789);
// impl_rng_integer!(usize, 0xABCD_EF01);

// Implement for common signed integer types
impl_rng_integer!(i8, 0x3B);
impl_rng_integer!(i16, 0x3BCD);
// impl_rng_integer!(i32, 0x3BCD_EF01);
// impl_rng_integer!(i64, 0x3BCD_EF01_2345_6789);
// impl_rng_integer!(isize, 0x3BCD_EF01);
