//! Pools for arena allocations

use core::marker::PhantomData;
use crate::ArenaIndex;

#[derive(Debug, Clone, Copy)]
pub struct Pool<T, SizeType = u16, Marker = ()> {
    pub(crate) offset: SizeType,
    pub(crate) len: SizeType,
    pub(crate) generation: u16,
    pub(crate) arena_id: u16,
    pub(crate) _phantom: PhantomData<(T, Marker)>,
}

impl<T, SizeType, Marker> Pool<T, SizeType, Marker> {
    /// Create a new pool (internal use)
    pub(crate) fn new(offset: SizeType, len: SizeType, generation: u16, arena_id: u16) -> Self {
        Self {
            offset,
            len,
            generation,
            arena_id,
            _phantom: PhantomData
        }
    }

    /// Get element count
    pub fn len(&self) -> SizeType
    where
        SizeType: ArenaIndex,
    {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool
    where
        SizeType: ArenaIndex,
    {
        self.len.into() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> SizeType
    where
        SizeType: ArenaIndex,
    {
        self.offset
    }

    /// Get generation
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Get size in bytes
    pub fn size_bytes(&self) -> usize
    where
        SizeType: ArenaIndex,
    {
        self.len.into() * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> (usize, usize)
    where
        SizeType: ArenaIndex,
    {
        (self.len.into(), self.len.into())
    }
}

impl<T, SizeType, Marker> Default for Pool<T, SizeType, Marker>
where
    SizeType: ArenaIndex,
{
    fn default() -> Self {
        Self {
            offset: SizeType::zero(),
            len: SizeType::zero(),
            generation: 0,
            arena_id: 0,
            _phantom: PhantomData,
        }
    }
}
