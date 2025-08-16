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
    Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = Self> + tato_math::Num
{
}

// Implement for types that can safely convert to usize on all platforms
impl ArenaIndex for u8 {}
impl ArenaIndex for u16 {}
// impl ArenaIndex for u32 {}
impl ArenaIndex for usize {}

#[cfg(test)]
mod tests;
