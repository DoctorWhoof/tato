#![no_std]
//! Fixed-size, no_std arena allocator with generational safety
//! and type markers for static safety.

pub mod arena;
pub mod buffer;
pub mod id;
pub mod slice;
pub mod text;
// pub mod typed_arena;

pub use arena::Arena;
pub use buffer::*;
pub use id::{ArenaId, RawId};
pub use slice::Slice;
pub use text::*;
// pub use typed_arena::{TypedArena, TypedId};

/// Trait for types that can be used as arena indices.
/// This consolidates all the requirements for index types like u8, u16, usize.
pub trait ArenaIndex:
    Copy + TryFrom<usize> + PartialOrd + core::ops::Add<Output = Self> + tato_math::Num
{
    fn to_usize(self) -> usize;
}

// Implement for types that can safely convert to usize on all platforms
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod u32_tests {
    use super::*;

    #[test]
    fn test_u32_arena_index() {
        let mut arena: Arena<1024, u32> = Arena::new();

        // Test basic allocation with u32 indices
        let id1 = arena.alloc(42i32).expect("Failed to allocate");
        let id2 = arena.alloc(100i32).expect("Failed to allocate");

        // Test retrieval
        assert_eq!(*arena.get(&id1).unwrap(), 42);
        assert_eq!(*arena.get(&id2).unwrap(), 100);

        // Test that the indices can be converted to usize
        assert!(id1.offset() < 1024);
        assert!(id2.offset() < 1024);

        // Test slice allocation
        let slice_id = arena.alloc_slice_from_fn(3u32, |i| i as u8).expect("Failed to allocate slice");
        let slice = arena.get_slice(&slice_id).unwrap();
        assert_eq!(slice, &[0u8, 1u8, 2u8]);
    }
}
