use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

// Operations we need for a PixelCluster to be able to use multiple
// cluster sizes (will be useful in the future!)
pub trait IntegerOps:
    Sized + Copy + Default + PartialEq +
    BitAnd<Output = Self> + BitOr<Output = Self> +
    Not<Output = Self> + Shl<usize, Output = Self> +
    Shr<usize, Output = Self>
{
    const BITS: usize;
    fn from_u8(val: u8) -> Self;
    fn to_u8(self) -> u8;
}

// Implement for common integer types
impl IntegerOps for u8 {
    const BITS: usize = 8;
    #[inline(always)] fn from_u8(val: u8) -> Self { val }
    #[inline(always)] fn to_u8(self) -> u8 { self }
}

impl IntegerOps for u16 {
    const BITS: usize = 16;
    #[inline(always)] fn from_u8(val: u8) -> Self { val as u16 }
    #[inline(always)] fn to_u8(self) -> u8 { self as u8 }
}

impl IntegerOps for u32 {
    const BITS: usize = 32;
    #[inline(always)] fn from_u8(val: u8) -> Self { val as u32 }
    #[inline(always)] fn to_u8(self) -> u8 { self as u8 }
}

impl IntegerOps for u64 {
    const BITS: usize = 64;
    #[inline(always)] fn from_u8(val: u8) -> Self { val as u64 }
    #[inline(always)] fn to_u8(self) -> u8 { self as u8 }
}
