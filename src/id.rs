//! Arena ID - Type-safe handles to allocated values
//!
//! This module provides the `ArenaId<T>` type, which serves as a handle to
//! values allocated in the arena. It's designed to be small, efficient, and
//! type-safe while avoiding runtime overhead.
//!
//! # Design Philosophy
//!
//! Following old-school embedded principles:
//! - Small handle size (16 bytes on 64-bit systems)
//! - Compile-time type safety
//! - Zero runtime type checking overhead
//! - Direct memory access via offset + size

use core::marker::PhantomData;

/// A handle to an allocated value in the arena
///
/// This is intentionally kept small and relies on compile-time
/// type safety rather than runtime type checking for performance.
/// Perfect for old-school programming where every byte and cycle counts.
///
/// # Memory Layout
///
/// ```text
/// ArenaId<T> {
///     offset: usize,    // 8 bytes (64-bit)
///     size: usize,      // 8 bytes (64-bit)
///     _marker: PhantomData<T>, // 0 bytes
/// }
/// Total: 16 bytes
/// ```
///
/// # Safety
///
/// ArenaId is safe to use as long as:
/// - The arena that created it is still alive
/// - The arena hasn't been cleared since allocation
/// - The type T matches the original allocation type
///
/// The type system enforces the last point at compile time.
#[derive(Debug, Clone, Copy)]
pub struct ArenaId<T> {
    /// Offset within the arena's storage
    pub(crate) offset: usize,
    /// Size of the allocation in bytes
    pub(crate) size: usize,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _marker: PhantomData<T>,
}

impl<T> ArenaId<T> {
    /// Create a new ArenaId with the given offset and size
    ///
    /// This is primarily used internally by the arena allocator.
    pub(crate) fn new(offset: usize, size: usize) -> Self {
        Self {
            offset,
            size,
            _marker: PhantomData,
        }
    }

    /// Get the offset of this allocation within the arena
    ///
    /// This is the byte offset from the start of the arena's storage
    /// where this value is located.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the size of this allocation in bytes
    ///
    /// This is the actual size of the allocated type T.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the type information as a tuple (offset, size)
    ///
    /// Useful for debugging and introspection.
    pub fn info(&self) -> (usize, usize) {
        (self.offset, self.size)
    }

    /// Check if this ID represents a valid allocation
    ///
    /// An ID is considered valid if it has non-zero size.
    /// Note: This doesn't check if the arena is still alive.
    pub fn is_valid(&self) -> bool {
        self.size > 0
    }
}

/// Equality comparison for ArenaId
///
/// Two ArenaIds are equal if they point to the same location
/// and have the same size. The type is enforced at compile time.
impl<T> PartialEq for ArenaId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.size == other.size
    }
}

impl<T> Eq for ArenaId<T> {}

/// Hash implementation for ArenaId
///
/// Allows ArenaId to be used as keys in hash maps/sets.
impl<T> core::hash::Hash for ArenaId<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.size.hash(state);
    }
}

