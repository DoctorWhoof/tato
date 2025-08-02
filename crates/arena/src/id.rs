//! Arena ID - Handle to values allocated in the arena.

use core::marker::PhantomData;
use crate::ArenaIndex;

/// Type-erased arena handle. Use `typed()` to convert back.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RawId<SizeType = u16> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Size of the original type in bytes (for type checking)
    pub(crate) type_size: SizeType,
    /// Generation when this ID was created
    pub(crate) generation: u16,
    /// Arena ID for cross-arena safety
    pub(crate) arena_id: u16,
}

impl<SizeType> RawId<SizeType>
where
    SizeType: ArenaIndex + PartialEq,
{
    /// Convert to typed ID. Panics in debug if size mismatch.
    /// Will NOT catch all problems, i.e. if types are different but have same size.
    pub fn typed<T, Marker>(self) -> ArenaId<T, SizeType, Marker> {
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
            generation: self.generation,
            arena_id: self.arena_id,
            _phantom: PhantomData,
        }
    }

    /// Get generation
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }
}

/// Handle to a value in the arena
#[derive(Debug, Clone, Copy, Hash)]
pub struct ArenaId<T, SizeType = u16, Marker = ()> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Generation when this ID was created
    pub(crate) generation: u16,
    /// Arena ID for cross-arena safety
    pub(crate) arena_id: u16,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _phantom: PhantomData<(T, Marker)>,
}

impl<T, SizeType, Marker> ArenaId<T, SizeType, Marker> {
    /// Create a new ArenaId (internal use)
    pub(crate) fn new(offset: SizeType, size: SizeType, generation: u16, arena_id: u16) -> Self {
        Self {
            offset,
            size,
            generation,
            arena_id,
            _phantom: PhantomData,
        }
    }

    /// Get byte offset in arena
    pub fn offset(&self) -> usize
    where
        SizeType: ArenaIndex,
    {
        self.offset.into()
    }

    /// Get allocation size in bytes
    pub fn size(&self) -> usize
    where
        SizeType: ArenaIndex,
    {
        self.size.into()
    }

    /// Get generation
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Get (offset, size) tuple
    pub fn info(&self) -> (usize, usize)
    where
        SizeType: ArenaIndex,
    {
        (self.offset.into(), self.size.into())
    }

    /// Check if ID has non-zero size
    pub fn is_valid(&self) -> bool
    where
        SizeType: ArenaIndex,
    {
        self.size.into() > 0
    }

    /// Convert to type-erased RawId
    pub fn raw(self) -> RawId<SizeType>
    where
        SizeType: ArenaIndex,
    {
        RawId {
            offset: self.offset,
            size: self.size,
            type_size: SizeType::try_from(core::mem::size_of::<T>())
                .unwrap_or_else(|_| panic!("Type size too large for SizeType")),
            generation: self.generation,
            arena_id: self.arena_id,
        }
    }
}

/// ArenaIds are equal if they have the same offset, size, and generation
impl<T, SizeType, Marker> PartialEq for ArenaId<T, SizeType, Marker>
where
    SizeType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.size == other.size
            && self.generation == other.generation
            && self.arena_id == other.arena_id
    }
}

impl<T, SizeType, Marker> Eq for ArenaId<T, SizeType, Marker>
where
    SizeType: Eq,
{}
