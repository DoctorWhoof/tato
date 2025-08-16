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

impl<T, Idx, Marker> Slice<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    /// Create a new Slice (internal use)
    pub(crate) fn new(offset: Idx, len: Idx, generation: u16, arena_id: u16) -> Self {
        Self { offset, len, generation, arena_id, _phantom: PhantomData }
    }

    /// Get element count
    pub fn len(&self) -> Idx {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len.into() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> Idx {
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
    pub fn size_bytes(&self) -> usize {
        self.len.into() * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> Idx {
        self.len
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
