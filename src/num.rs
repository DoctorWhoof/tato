//! Number traits to allow generic code to use any number type.
//! Does not cover all cases, only what this crate needs!

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
// use core::ops::Neg;

/// A Number trait.
/// Automatically implemented for primitive number types (i32, f32, etc).
pub trait Num:
    Default
    + PartialEq
    + PartialOrd
    + Copy
    + AddAssign
    + MulAssign
    + SubAssign
    + DivAssign
    + Add<Output = Self>
    + Mul<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self;
    fn four() -> Self;
    // fn from_usize(value: usize) -> Self;
    fn get_max(self, b: Self) -> Self;
    fn get_min(self, b: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_add(self, rhs: Self) -> Self;
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
    // fn round_up(self) -> Self;
    // fn round_down(self) -> Self;
}

// /// A signed Number trait.
// pub trait SignedNum: Num + Neg<Output = Self> {}

// Private module for implementation details
mod macros {
    use super::Num;
    /// Implements Num for unsigned integer types
    macro_rules! impl_uint_num {
        ($t:ty) => {
            impl Num for $t {
                #[inline(always)]
                fn zero() -> Self {
                    0
                }

                #[inline(always)]
                fn one() -> Self {
                    1
                }

                #[inline(always)]
                fn two() -> Self {
                    2
                }

                #[inline(always)]
                fn four() -> Self {
                    4
                }

                // #[inline(always)]
                // fn max_value() -> Self {
                //     Self::MAX
                // }

                #[inline(always)]
                fn get_max(self, b: Self) -> Self {
                    self.max(b)
                }

                #[inline(always)]
                fn get_min(self, b: Self) -> Self {
                    self.min(b)
                }

                #[inline(always)]
                fn saturating_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }

                #[inline(always)]
                fn saturating_add(self, rhs: Self) -> Self {
                    self.saturating_add(rhs)
                }

                #[inline(always)]
                fn from_f32(value: f32) -> Self {
                    // Round half to even (banker's rounding) for unsigned integers
                    // Clamp negative values to 0
                    if value < 0.0 {
                        0
                    } else {
                        let truncated = value as i64;
                        let frac = value - (truncated as f32);

                        if frac > 0.5 {
                            (truncated + 1) as Self
                        } else if frac < 0.5 {
                            truncated as Self
                        } else {
                            // Exactly 0.5 - round to even
                            if truncated % 2 == 0 {
                                truncated as Self
                            } else {
                                (truncated + 1) as Self
                            }
                        }
                    }
                }

                #[inline(always)]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                // #[inline(always)]
                // fn round_up(self) -> Self {
                //     self
                // }

                // fn round_down(self) -> Self {
                //     self
                // }
            }
        };
    }

    /// Implements Num for signed integer types
    macro_rules! impl_sint_num {
        ($t:ty) => {
            impl Num for $t {
                #[inline(always)]
                fn zero() -> Self {
                    0
                }

                #[inline(always)]
                fn one() -> Self {
                    1
                }

                #[inline(always)]
                fn two() -> Self {
                    2
                }

                #[inline(always)]
                fn four() -> Self {
                    4
                }

                // #[inline(always)]
                // fn max_value() -> Self {
                //     Self::MAX
                // }

                #[inline(always)]
                fn get_max(self, b: Self) -> Self {
                    self.max(b)
                }

                #[inline(always)]
                fn get_min(self, b: Self) -> Self {
                    self.min(b)
                }

                #[inline(always)]
                fn saturating_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }

                #[inline(always)]
                fn saturating_add(self, rhs: Self) -> Self {
                    self.saturating_add(rhs)
                }

                #[inline(always)]
                fn from_f32(value: f32) -> Self {
                    // Round half to even (banker's rounding) for signed integers
                    let truncated = value as i64;
                    let frac = if value >= 0.0 {
                        value - (truncated as f32)
                    } else {
                        (truncated as f32) - value
                    };

                    if frac > 0.5 {
                        if value >= 0.0 {
                            (truncated + 1) as Self
                        } else {
                            (truncated - 1) as Self
                        }
                    } else if frac < 0.5 {
                        truncated as Self
                    } else {
                        // Exactly 0.5 - round to even
                        if truncated % 2 == 0 {
                            truncated as Self
                        } else {
                            if value >= 0.0 {
                                (truncated + 1) as Self
                            } else {
                                (truncated - 1) as Self
                            }
                        }
                    }
                }

                #[inline(always)]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                // #[inline(always)]
                // fn round_up(self) -> Self {
                //     self
                // }

                // fn round_down(self) -> Self {
                //     self
                // }
            }
        };
    }

    // /// Takes in the type and the necessary exponential function for that type.
    // macro_rules! impl_signed_num {
    //     ($t:ty) => {
    //         impl SignedNum for $t {}
    //     };
    // }

    /// Implements Num for float types
    macro_rules! impl_float_num {
        ($t:ty) => {
            impl Num for $t {
                #[inline(always)]
                fn zero() -> Self {
                    0.0
                }

                #[inline(always)]
                fn one() -> Self {
                    1.0
                }

                #[inline(always)]
                fn two() -> Self {
                    2.0
                }

                #[inline(always)]
                fn four() -> Self {
                    4.0
                }

                // #[inline(always)]
                // fn max_value() -> Self {
                //     Self::INFINITY
                // }

                // #[inline(always)]
                // fn from_usize(value: usize) -> Self {
                //     value as Self
                // }

                #[inline(always)]
                fn get_max(self, b: Self) -> Self {
                    self.max(b)
                }

                #[inline(always)]
                fn get_min(self, b: Self) -> Self {
                    self.min(b)
                }

                #[inline(always)]
                fn saturating_sub(self, rhs: Self) -> Self {
                    if rhs > self { Self::zero() } else { self - rhs }
                }

                #[inline(always)]
                fn saturating_add(self, rhs: Self) -> Self {
                    self + rhs
                }

                #[inline(always)]
                fn from_f32(value: f32) -> Self {
                    value as Self
                }

                #[inline(always)]
                fn to_f32(self) -> f32 {
                    self as f32
                }

                // #[inline(always)]
                // fn round_down(self) -> Self {
                //     // Floor function: largest integer <= self
                //     let truncated = self as i64 as Self;
                //     if self >= 0.0 || self == truncated {
                //         truncated
                //     } else {
                //         truncated - 1.0
                //     }
                // }

                // #[inline(always)]
                // fn round_up(self) -> Self {
                //     // Ceil function: smallest integer >= self
                //     let truncated = self as i64 as Self;
                //     if self <= 0.0 || self == truncated {
                //         truncated
                //     } else {
                //         truncated + 1.0
                //     }
                // }
            }
        };
    }

    impl_uint_num!(u8);
    impl_uint_num!(u16);
    impl_uint_num!(u32);
    impl_uint_num!(u64);
    impl_uint_num!(usize);

    impl_sint_num!(i8);
    impl_sint_num!(i16);
    impl_sint_num!(i32);
    impl_sint_num!(i64);
    impl_sint_num!(isize);

    impl_float_num!(f32);
    impl_float_num!(f64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_rounding() {
        // basic rounding
        assert_eq!(u32::from_f32(3.4), 3);
        assert_eq!(u32::from_f32(3.6), 4);
        assert_eq!(u32::from_f32(4.0), 4);

        // banker's rounding (round half to even)
        assert_eq!(u32::from_f32(2.5), 2);
        assert_eq!(u32::from_f32(3.5), 4);
        assert_eq!(u32::from_f32(4.5), 4);
        assert_eq!(u32::from_f32(5.5), 6);

        // negative values get clamped to 0
        assert_eq!(u32::from_f32(-1.5), 0);
        assert_eq!(u32::from_f32(-0.1), 0);

        // edge cases
        assert_eq!(u16::from_f32(0.4), 0);
        assert_eq!(u16::from_f32(0.5), 0);
        assert_eq!(u16::from_f32(1.5), 2);
    }

    #[test]
    fn test_signed_rounding() {
        // basic rounding
        assert_eq!(i32::from_f32(3.4), 3);
        assert_eq!(i32::from_f32(3.6), 4);
        assert_eq!(i32::from_f32(-3.4), -3);
        assert_eq!(i32::from_f32(-3.6), -4);

        // banker's rounding (round half to even)
        assert_eq!(i32::from_f32(2.5), 2);
        assert_eq!(i32::from_f32(3.5), 4);
        assert_eq!(i32::from_f32(4.5), 4);
        assert_eq!(i32::from_f32(5.5), 6);
        assert_eq!(i32::from_f32(-2.5), -2);
        assert_eq!(i32::from_f32(-3.5), -4);
        assert_eq!(i32::from_f32(-4.5), -4);
        assert_eq!(i32::from_f32(-5.5), -6);

        // edge cases
        assert_eq!(i16::from_f32(0.4), 0);
        assert_eq!(i16::from_f32(0.5), 0);
        assert_eq!(i16::from_f32(1.5), 2);
        assert_eq!(i16::from_f32(-0.4), 0);
        assert_eq!(i16::from_f32(-0.5), 0);
        assert_eq!(i16::from_f32(-1.5), -2);
    }

    #[test]
    fn test_float_passthrough() {
        // Float types should pass through unchanged
        assert_eq!(f32::from_f32(3.7), 3.7);
        assert_eq!(f32::from_f32(-2.3), -2.3);
        assert_eq!(f64::from_f32(1.5), 1.5);
    }

    #[test]
    fn test_to_f32_conversion() {
        assert_eq!(42u32.to_f32(), 42.0);
        assert_eq!((-17i32).to_f32(), -17.0);
        assert_eq!(3.14f32.to_f32(), 3.14);
    }

    #[test]
    fn test_accumulated_rounding_bias() {
        // Test that banker's rounding reduces bias in accumulated operations
        // This simulates repeated layout calculations with values that would
        // accumulate bias with simple "round half up" rounding
        let test_values = [2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5];

        // With banker's rounding, half the .5 values round up, half round down
        let banker_sum: i32 = test_values.iter().map(|&v| i32::from_f32(v)).sum();

        // Expected: 2 + 4 + 4 + 6 + 6 + 8 + 8 + 10 = 48
        // This is exactly the sum of the original values (44) rounded to nearest int
        assert_eq!(banker_sum, 48);

        // Verify individual conversions for clarity
        assert_eq!(i32::from_f32(2.5), 2); // round down to even
        assert_eq!(i32::from_f32(3.5), 4); // round up to even
        assert_eq!(i32::from_f32(4.5), 4); // round down to even
        assert_eq!(i32::from_f32(5.5), 6); // round up to even

        // Test with small pixel-like values
        let pixel_values = [0.5, 1.5, 2.5, 3.5];
        let pixel_sum: u32 = pixel_values.iter().map(|&v| u32::from_f32(v)).sum();

        // Expected: 0 + 2 + 2 + 4 = 8 (perfect balance)
        assert_eq!(pixel_sum, 8);
    }
}
