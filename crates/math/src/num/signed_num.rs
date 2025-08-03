use core::ops::Neg;
use super::num::Num;

/// A trait for numeric types that support negation (signed integers and floats).
pub trait SignedNum: Num + Neg<Output = Self> {}