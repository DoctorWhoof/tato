//! Arena ID - Handle to values allocated in the arena.

use core::marker::PhantomData;

/// Type-erased arena handle. Use `typed()` to convert back.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RawId<SizeType = usize> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Size of the original type in bytes (for type checking)
    pub(crate) type_size: SizeType,
}

impl<SizeType> RawId<SizeType>
where
    SizeType: Copy + PartialEq + TryFrom<usize> + Into<usize>,
{
    /// Convert to typed ID. Panics in debug if size mismatch.
    /// Will NOT catch all problems, i.e. if types are different but have same size.
    pub fn typed<T>(self) -> ArenaId<T, SizeType> {
        let expected_size = core::mem::size_of::<T>();
        let stored_size: usize = self.type_size.into();
        debug_assert_eq!(
            stored_size,
            expected_size,
            "Type size mismatch: attempted to convert RawId to wrong type. \
             Expected size: {}, but T has size: {}",
            stored_size,
            expected_size
        );

        ArenaId {
            offset: self.offset,
            size: self.size,
            _marker: PhantomData,
        }
    }
}

/// Handle to a value in the arena
#[derive(Debug, Clone, Copy, Hash)]
pub struct ArenaId<T, SizeType = usize> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _marker: PhantomData<T>,
}

impl<T, SizeType> ArenaId<T, SizeType> {
    /// Create a new ArenaId (internal use)
    pub(crate) fn new(offset: SizeType, size: SizeType) -> Self {
        Self {
            offset,
            size,
            _marker: PhantomData,
        }
    }

    /// Get byte offset in arena
    pub fn offset(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.offset.into()
    }

    /// Get allocation size in bytes
    pub fn size(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into()
    }

    /// Get (offset, size) tuple
    pub fn info(&self) -> (usize, usize)
    where
        SizeType: Copy + Into<usize>,
    {
        (self.offset.into(), self.size.into())
    }

    /// Check if ID has non-zero size
    pub fn is_valid(&self) -> bool
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into() > 0
    }

    /// Convert to type-erased RawId
    pub fn raw(self) -> RawId<SizeType>
    where
        SizeType: TryFrom<usize>,
    {
        RawId {
            offset: self.offset,
            size: self.size,
            type_size: SizeType::try_from(core::mem::size_of::<T>())
                .unwrap_or_else(|_| panic!("Type size too large for SizeType")),
        }
    }
}

/// ArenaIds are equal if they have the same offset and size
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

// /// Hash implementation for ArenaId
// ///
// /// Allows ArenaId to be used as keys in hash maps/sets.
// impl<T, SizeType> core::hash::Hash for ArenaId<T, SizeType>
// where
//     SizeType: core::hash::Hash,
// {
//     fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
//         self.offset.hash(state);
//         self.size.hash(state);
//     }
// }
