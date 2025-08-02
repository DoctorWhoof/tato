#![no_std]
//! Fixed-size arena allocator with generational safety and type markers.

pub mod arena;
pub mod id;
pub mod pool;

pub use arena::Arena;
pub use id::{ArenaId, RawId};
pub use pool::Pool;

/// Trait for types that can be used as arena indices.
/// 
/// This consolidates all the requirements for index types like u8, u16, usize.
pub trait ArenaIndex: 
    Copy + 
    TryFrom<usize> + 
    Into<usize> + 
    PartialOrd + 
    core::ops::Add<Output = Self> +
    tato_math::Num
{
}

// Implement for types that can safely convert to usize on all platforms
impl ArenaIndex for u8 {}
impl ArenaIndex for u16 {}
impl ArenaIndex for usize {}

#[cfg(test)]
mod tests;
