//! Number traits to allow generic code to use any number type.

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// A Number trait, includes signed and unsigned but can't be negated. Use [SignedNum] for that.
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
    fn max() -> Self;
    fn from_usize(value: usize) -> Self;
    fn get_max(a: Self, b: Self) -> Self;
    fn get_min(a: Self, b: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn saturating_add(self, rhs: Self) -> Self;
    fn from_f32(value:f32) -> Self;
    fn to_f32(self) -> f32;
}

/// A signed Number trait.
pub trait SignedNum: Num + Neg<Output = Self> {}

/// Implements Num for integer types
#[macro_export]
macro_rules! impl_int_num {
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
            fn max() -> Self {
                Self::MAX
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Self {
                value as Self
            }

            #[inline(always)]
            fn get_max(a: Self, b: Self) -> Self {
                a.max(b)
            }

            #[inline(always)]
            fn get_min(a: Self, b: Self) -> Self {
                a.min(b)
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
                value as Self
            }

            #[inline(always)]
            fn to_f32(self) -> f32 {
                self as f32
            }
        }
    };
}

/// Takes in the type and the necessary exponential function for that type.
#[macro_export]
macro_rules! impl_signed_num {
    ($t:ty) => {
        impl SignedNum for $t {}
    };
}

/// Implements Num for float types
#[macro_export]
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
            fn max() -> Self {
                Self::INFINITY
            }

            #[inline(always)]
            fn from_usize(value: usize) -> Self {
                value as Self
            }

            #[inline(always)]
            fn get_max(a: Self, b: Self) -> Self {
                a.max(b)
            }

            #[inline(always)]
            fn get_min(a: Self, b: Self) -> Self {
                a.min(b)
            }

            #[inline(always)]
            fn saturating_sub(self, rhs: Self) -> Self {
                if rhs > self {
                    Self::zero()
                } else {
                    self - rhs
                }
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

impl_int_num!(u8);
impl_int_num!(u16);
impl_int_num!(u32);
impl_int_num!(u64);
impl_int_num!(usize);

impl_int_num!(i8);
impl_int_num!(i16);
impl_int_num!(i32);
impl_int_num!(i64);
impl_int_num!(isize);

impl_float_num!(f32);
impl_float_num!(f64);

impl_signed_num!(i8);
impl_signed_num!(i16);
impl_signed_num!(i32);
impl_signed_num!(i64);
impl_signed_num!(isize);
impl_signed_num!(f32);
impl_signed_num!(f64);
