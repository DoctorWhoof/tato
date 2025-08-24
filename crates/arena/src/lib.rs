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

/// Error type for arena operations that can fail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaError {
    /// Not enough space in the arena for the requested allocation
    OutOfSpace { requested: usize, available: usize },
    /// Attempt to access an ID with mismatched generation (temporal safety violation)
    InvalidGeneration { expected: u32, found: u32 },
    /// Attempt to access an ID from a different arena (cross-arena safety violation)
    CrossArenaAccess { expected_id: u16, found_id: u16 },
    /// Invalid bounds or slice range
    InvalidBounds,
    /// Index type conversion failed
    IndexConversion,
    /// Slice capacity exceeded
    CapacityExceeded,
    /// Text generation failed due to empty source bytes or invalid characters
    InvalidOrEmptyUTF8
}

impl core::fmt::Display for ArenaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ArenaError::OutOfSpace { requested, available } => {
                write!(f, "Arena out of space: requested {} bytes, {} available", requested, available)
            }
            ArenaError::InvalidGeneration { expected, found } => {
                write!(f, "Invalid generation: expected {}, found {}", expected, found)
            }
            ArenaError::CrossArenaAccess { expected_id, found_id } => {
                write!(f, "Cross-arena access: expected arena {}, found {}", expected_id, found_id)
            }
            ArenaError::InvalidBounds => write!(f, "Invalid bounds or range"),
            ArenaError::IndexConversion => write!(f, "Index type conversion failed"),
            ArenaError::CapacityExceeded => write!(f, "Capacity exceeded"),
            ArenaError::InvalidOrEmptyUTF8 => write!(f, "Invalid or empty UTF8"),
        }
    }
}

/// Result type for arena operations
pub type ArenaResult<T> = Result<T, ArenaError>;

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
        assert_eq!(*arena.get(&id1).expect("Failed to get value"), 42);
        assert_eq!(*arena.get(&id2).expect("Failed to get value"), 100);

        // Test that the indices can be converted to usize
        assert!(id1.offset() < 1024);
        assert!(id2.offset() < 1024);

        // Test slice allocation
        let slice_id = arena.alloc_slice_from_fn(3u32, |i| i as u8).expect("Failed to allocate slice");
        let slice = arena.get_slice(&slice_id).expect("Failed to get slice");
        assert_eq!(slice, &[0u8, 1u8, 2u8]);
    }
}
