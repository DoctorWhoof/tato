#![no_std]

//! A very lightweight temporary arena allocator for values that live for a specific
//! scope or iteration (like a game frame).
//!
//! ```
//! {
//!     use temp_arena::TempArena;
//!
//!     // Create a new arena for this frame/iteration
//!     let mut frame_arena = TempArena::<4096>::new();
//!
//!     // Allocate temporary data
//!     let id1 = frame_arena.alloc(42i32).unwrap();
//!     let slice_id = frame_arena.alloc_slice_from_fn(10, |i| i * 2).unwrap();
//!
//!     // Use the data throughout this scope
//!     let value = frame_arena.get(id1).unwrap();
//!     let slice = frame_arena.get_slice(slice_id).unwrap();
//!
//!     // Modify slice data directly (returns &mut [T])
//!     let slice_mut = frame_arena.get_slice_mut(slice_id).unwrap();
//!     slice_mut[0] = 999;
//!
//!     // Arena automatically drops at end of scope, all IDs become invalid
//! }
//! ```
//! Unlike traditional arenas, it doesn't have a `clear()` function. Instead,
//! create a new arena for each scope. This ensures memory safety: when
//! the arena drops, all IDs become naturally invalid, preventing use-after-free bugs.
//!
//! All allocations in TempArena are **always initialized** for simplicity.

mod arena;
pub use arena::*;

mod buffer;
pub use buffer::*;

mod text;
pub use text::*;

mod id;
use id::*;

mod index;
use index::*;

#[cfg(test)]
mod tests;

use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr;
use core::slice;

/// Error type for temp arena operations that can fail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempArenaError {
    /// Not enough space in the arena for the requested allocation
    OutOfSpace { requested: usize, available: usize },
    /// Index type conversion failed
    IndexConversion,
    /// Invalid bounds or range
    InvalidBounds,
    /// Buffer is at full capacity
    BufferFull,
}

impl core::fmt::Display for TempArenaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TempArenaError::OutOfSpace { requested, available } => {
                write!(
                    f,
                    "Arena out of space: requested {} bytes, {} available",
                    requested, available
                )
            },
            TempArenaError::IndexConversion => write!(f, "Index type conversion failed"),
            TempArenaError::InvalidBounds => write!(f, "Invalid bounds or range"),
            TempArenaError::BufferFull => write!(f, "Buffer is at full capacity"),
        }
    }
}

/// Result type for temp arena operations
pub type TempArenaResult<T> = Result<T, TempArenaError>;
