//! Arena ID - Handle to values allocated in the arena.

use crate::ArenaIndex;
use core::marker::PhantomData;

/// Type-erased arena handle. Use `typed()` to convert back.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RawId<Idx = u32> {
    /// Offset within the arena's storage
    pub(crate) offset: Idx,
    /// Size of the allocation in bytes
    pub(crate) size: Idx,
    /// Size of the original type in bytes (for type checking)
    pub(crate) type_size: Idx,
    /// Generation when this ID was created
    pub(crate) generation: u32,
    /// Arena ID for cross-arena safety
    pub(crate) arena_id: u16,
}

impl<Idx> RawId<Idx>
where
    Idx: ArenaIndex + PartialEq,
{
    /// Convert to typed ID. Panics in debug if size mismatch.
    /// Will NOT catch all problems, i.e. if types are different but have same size.
    pub fn typed<T, Marker>(self) -> ArenaId<T, Idx, Marker> {
        let expected_size = core::mem::size_of::<T>();
        let stored_size: usize = self.type_size.to_usize();
        debug_assert_eq!(
            stored_size, expected_size,
            "Type size mismatch: attempted to convert RawId to wrong type. \
             Expected size: {}, but T has size: {}",
            stored_size, expected_size
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
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }
}

/// Handle to a value in the arena
#[derive(Debug, Clone, Copy, Hash)]
pub struct ArenaId<T, Idx = u32, Marker = ()> {
    /// Offset within the arena's storage
    pub(crate) offset: Idx,
    /// Size of the allocation in bytes
    pub(crate) size: Idx,
    /// Generation when this ID was created
    pub(crate) generation: u32,
    /// Arena ID for cross-arena safety
    pub(crate) arena_id: u16,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _phantom: PhantomData<(T, Marker)>,
}

impl<T, Idx, Marker> ArenaId<T, Idx, Marker> {
    /// Create a new ArenaId (internal use)
    pub(crate) fn new(offset: Idx, size: Idx, generation: u32, arena_id: u16) -> Self {
        Self { offset, size, generation, arena_id, _phantom: PhantomData }
    }

    /// Get byte offset in arena
    pub fn offset(self) -> usize
    where
        Idx: ArenaIndex,
    {
        self.offset.to_usize()
    }

    /// Get allocation size in bytes
    pub fn size(self) -> usize
    where
        Idx: ArenaIndex,
    {
        self.size.to_usize()
    }

    /// Get generation
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Get (offset, size) tuple
    pub fn info(self) -> (usize, usize)
    where
        Idx: ArenaIndex,
    {
        (self.offset.to_usize(), self.size.to_usize())
    }

    /// Check if ID has non-zero size
    pub fn is_valid(self) -> bool
    where
        Idx: ArenaIndex,
    {
        self.size.to_usize() > 0
    }

    /// Convert to type-erased RawId
    pub fn raw(self) -> RawId<Idx>
    where
        Idx: ArenaIndex,
    {
        RawId {
            offset: self.offset,
            size: self.size,
            type_size: Idx::try_from(core::mem::size_of::<T>())
                .unwrap_or_else(|_| panic!("Type size too large for Idx")),
            generation: self.generation,
            arena_id: self.arena_id,
        }
    }
}

/// ArenaIds are equal if they have the same offset, size, and generation
impl<T, Idx, Marker> PartialEq for ArenaId<T, Idx, Marker>
where
    Idx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && self.size == other.size
            && self.generation == other.generation
            && self.arena_id == other.arena_id
    }
}

impl<T, Idx, Marker> Eq for ArenaId<T, Idx, Marker> where Idx: Eq {}
