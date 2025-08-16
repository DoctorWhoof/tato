//! Pools for arena allocations

use crate::ArenaIndex;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Slice<T, Idx = u16, Marker = ()> {
    pub(crate) offset: Idx,
    pub(crate) len: Idx,
    pub(crate) generation: u16,
    pub(crate) arena_id: u16,
    pub(crate) _phantom: PhantomData<(T, Marker)>,
}

impl<T, Idx, Marker> Slice<T, Idx, Marker> {
    /// Create a new pool (internal use)
    pub(crate) fn new(offset: Idx, len: Idx, generation: u16, arena_id: u16) -> Self {
        Self { offset, len, generation, arena_id, _phantom: PhantomData }
    }

    /// Get element count
    pub fn len(&self) -> Idx
    where
        Idx: ArenaIndex,
    {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool
    where
        Idx: ArenaIndex,
    {
        self.len.into() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> Idx
    where
        Idx: ArenaIndex,
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
        Idx: ArenaIndex,
    {
        self.len.into() * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> (usize, usize)
    where
        Idx: ArenaIndex,
    {
        (self.len.into(), self.len.into())
    }
}

impl<T, Idx, Marker> Default for Slice<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self {
            offset: Idx::zero(),
            len: Idx::zero(),
            generation: 0,
            arena_id: 0,
            _phantom: PhantomData,
        }
    }
}
