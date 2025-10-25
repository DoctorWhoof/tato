
/// Trait for types that can be used as arena indices.
pub trait ArenaIndex: Copy + TryFrom<usize> + PartialOrd {
    fn to_usize(self) -> usize;
}

impl ArenaIndex for u8 {
    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl ArenaIndex for u16 {
    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl ArenaIndex for u32 {
    #[inline(always)]
    fn to_usize(self) -> usize {
        self as usize
    }
}

impl ArenaIndex for usize {
    #[inline(always)]
    fn to_usize(self) -> usize {
        self
    }
}
