//! Number traits to allow generic code to use any number type.
//! Does not cover all cases!

use core::ops::Neg;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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
    fn from_usize_checked(value: usize) -> Option<Self>;
    // fn from_usize(value: usize) -> Self;
    fn get_max(self, b: Self) -> Self;
    fn get_min(self, b: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_add(self, rhs: Self) -> Self;
    fn from_f32(value: f32) -> Self;
    fn to_f32(self) -> f32;
}

/// A Float trait that narrows Num to floating point types only.
/// Automatically implemented for f32 and f64.
pub trait Float: Num + Neg<Output = Self> {
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    // fn round_up(self) -> Self;
    // fn round_down(self) -> Self;
    fn exp(self) -> Self;
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

                #[inline(always)]
                fn from_usize_checked(value: usize) -> Option<Self> {
                    if value <= Self::MAX as usize { Some(value as Self) } else { None }
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

                #[inline(always)]
                fn from_usize_checked(value: usize) -> Option<Self> {
                    if value <= Self::MAX as usize { Some(value as Self) } else { None }
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
                        if value >= 0.0 { (truncated + 1) as Self } else { (truncated - 1) as Self }
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
            }
        };
    }


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

                #[inline(always)]
                fn from_usize_checked(value: usize) -> Option<Self> {
                    // For floats, conversion always succeeds but may lose precision
                    Some(value as Self)
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

    // Implement Float trait for floating point types
    impl super::Float for f32 {
        #[inline(always)]
        fn floor(self) -> Self {
            libm::floorf(self)
        }

        #[inline(always)]
        fn ceil(self) -> Self {
            libm::ceilf(self)
        }

        #[inline(always)]
        fn round(self) -> Self {
            libm::roundf(self)
        }

        #[inline(always)]
        fn exp(self) -> Self {
            libm::expf(self)
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

    impl super::Float for f64 {
        #[inline(always)]
        fn floor(self) -> Self {
            libm::floor(self)
        }

        #[inline(always)]
        fn ceil(self) -> Self {
            libm::ceil(self)
        }

        #[inline(always)]
        fn round(self) -> Self {
            libm::round(self)
        }

        #[inline(always)]
        fn exp(self) -> Self {
            libm::exp(self)
        }
    }
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

    #[test]
    fn test_from_usize_checked() {
        // Test successful conversions within range
        assert_eq!(u8::from_usize_checked(255), Some(255u8));
        assert_eq!(u8::from_usize_checked(0), Some(0u8));
        assert_eq!(u8::from_usize_checked(128), Some(128u8));

        assert_eq!(u16::from_usize_checked(65535), Some(65535u16));
        assert_eq!(u16::from_usize_checked(0), Some(0u16));
        assert_eq!(u16::from_usize_checked(32768), Some(32768u16));

        assert_eq!(i8::from_usize_checked(127), Some(127i8));
        assert_eq!(i8::from_usize_checked(0), Some(0i8));
        assert_eq!(i8::from_usize_checked(64), Some(64i8));

        assert_eq!(i16::from_usize_checked(32767), Some(32767i16));
        assert_eq!(i16::from_usize_checked(0), Some(0i16));
        assert_eq!(i16::from_usize_checked(16384), Some(16384i16));

        // Test failed conversions when value is too large
        assert_eq!(u8::from_usize_checked(256), None);
        assert_eq!(u8::from_usize_checked(1000), None);
        assert_eq!(u8::from_usize_checked(usize::MAX), None);

        assert_eq!(u16::from_usize_checked(65536), None);
        assert_eq!(u16::from_usize_checked(100000), None);

        assert_eq!(i8::from_usize_checked(128), None);
        assert_eq!(i8::from_usize_checked(256), None);

        assert_eq!(i16::from_usize_checked(32768), None);
        assert_eq!(i16::from_usize_checked(65536), None);

        // Test float conversions (always succeed)
        assert_eq!(f32::from_usize_checked(0), Some(0.0f32));
        assert_eq!(f32::from_usize_checked(1000000), Some(1000000.0f32));
        assert_eq!(f32::from_usize_checked(usize::MAX), Some(usize::MAX as f32));

        assert_eq!(f64::from_usize_checked(0), Some(0.0f64));
        assert_eq!(f64::from_usize_checked(1000000), Some(1000000.0f64));
        assert_eq!(f64::from_usize_checked(usize::MAX), Some(usize::MAX as f64));

        // Test larger integer types
        assert_eq!(u32::from_usize_checked(4294967295), Some(4294967295u32));
        assert_eq!(i32::from_usize_checked(2147483647), Some(2147483647i32));

        // Platform-dependent types
        assert_eq!(usize::from_usize_checked(1000), Some(1000usize));
        assert_eq!(isize::from_usize_checked(1000), Some(1000isize));
    }

    #[test]
    fn test_float_unary_negation() {
        // Test that Float trait supports unary negation
        fn negate_float<T: Float>(value: T) -> T {
            -value
        }

        // Test with f32
        assert_eq!(negate_float(3.14f32), -3.14f32);
        assert_eq!(negate_float(-2.5f32), 2.5f32);
        assert_eq!(negate_float(0.0f32), -0.0f32);

        // Test with f64
        assert_eq!(negate_float(2.71828f64), -2.71828f64);
        assert_eq!(negate_float(-1.414f64), 1.414f64);
        assert_eq!(negate_float(0.0f64), -0.0f64);

        // Test in generic context with other Num operations
        fn compute_with_negation<T: Float>(x: T, y: T) -> T {
            let neg_x = -x;
            let neg_y = -y;
            neg_x + neg_y + T::one()
        }

        assert_eq!(compute_with_negation(2.0f32, 3.0f32), -4.0f32);
        assert_eq!(compute_with_negation(1.5f64, 2.5f64), -3.0f64);
    }

    #[test]
    fn test_float_mathematical_functions() {
        // Test floor - rounds down
        assert_eq!(3.7f32.floor(), 3.0);
        assert_eq!((-3.7f32).floor(), -4.0);
        assert_eq!(5.0f64.floor(), 5.0); // Already integer

        // Test ceil - rounds up
        assert_eq!(3.2f32.ceil(), 4.0);
        assert_eq!((-3.2f64).ceil(), -3.0);
        assert_eq!(5.0f32.ceil(), 5.0); // Already integer

        // Test round - rounds to nearest (0.5 rounds away from zero)
        assert_eq!(3.4f32.round(), 3.0);
        assert_eq!(3.5f32.round(), 4.0);
        assert_eq!((-3.5f64).round(), -4.0);

        // Test exp - e^x
        let e = 2.718281828f32;
        assert!((1.0f32.exp() - e).abs() < 0.00001);
        assert!((0.0f64.exp() - 1.0).abs() < 0.000001);

        // Test in generic context
        fn test_generic_math<T: Float>(x: T) -> (T, T, T, T) {
            (x.floor(), x.ceil(), x.round(), x.exp())
        }

        let (f, c, r, _) = test_generic_math(3.7f32);
        assert_eq!(f, 3.0);
        assert_eq!(c, 4.0);
        assert_eq!(r, 4.0);
    }
}
