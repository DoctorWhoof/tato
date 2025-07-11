//! Arena ID - Handle to values allocated in the arena.

use core::marker::PhantomData;

/// A handle to an allocated value in the arena
#[derive(Debug, Clone, Copy)]
pub struct ArenaId<T, SizeType = usize> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _marker: PhantomData<T>,
}

impl<T, SizeType> ArenaId<T, SizeType> {
    /// Create a new ArenaId with the given offset and size
    ///
    /// This is primarily used internally by the arena allocator.
    pub(crate) fn new(offset: SizeType, size: SizeType) -> Self {
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
    pub fn offset(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.offset.into()
    }

    /// Get the size of this allocation in bytes
    ///
    /// This is the actual size of the allocated type T.
    pub fn size(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into()
    }

    /// Get the type information as a tuple (offset, size)
    ///
    /// Useful for debugging and introspection.
    pub fn info(&self) -> (usize, usize)
    where
        SizeType: Copy + Into<usize>,
    {
        (self.offset.into(), self.size.into())
    }

    /// Check if this ID represents a valid allocation
    ///
    /// An ID is considered valid if it has non-zero size.
    /// Note: This doesn't check if the arena is still alive.
    pub fn is_valid(&self) -> bool
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into() > 0
    }
}

/// Equality comparison for ArenaId
///
/// Two ArenaIds are equal if they point to the same location
/// and have the same size. The type is enforced at compile time.
impl<T, SizeType> PartialEq for ArenaId<T, SizeType>
where
    SizeType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.size == other.size
    }
}

impl<T, SizeType> Eq for ArenaId<T, SizeType>
where
    SizeType: Eq,
{}

/// Hash implementation for ArenaId
///
/// Allows ArenaId to be used as keys in hash maps/sets.
impl<T, SizeType> core::hash::Hash for ArenaId<T, SizeType>
where
    SizeType: core::hash::Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.offset.hash(state);
        self.size.hash(state);
    }
}
