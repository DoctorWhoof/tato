//! Pools for arena allocations

use crate::ArenaIndex;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Slice<T, I = u32, M = ()> {
    pub(crate) offset: I,
    pub(crate) len: I,
    pub(crate) generation: u32,
    pub(crate) arena_id: u16,
    pub(crate) _phantom: PhantomData<(T, M)>,
}

impl<T, I, M> Slice<T, I, M>
where
    I: ArenaIndex,
{
    /// Create a new Slice (internal use)
    pub(crate) fn new(offset: I, len: I, generation: u32, arena_id: u16) -> Self {
        Self { offset, len, generation, arena_id, _phantom: PhantomData }
    }

    /// Get element count
    pub fn len(&self) -> I {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len.to_usize() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> I {
        self.offset
    }

    /// Get generation
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        self.len.to_usize() * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> I {
        self.len
    }

    // Iterators

}

impl<T, I, M> Default for Slice<T, I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self {
            offset: I::zero(),
            len: I::zero(),
            generation: 0,
            arena_id: 0,
            _phantom: PhantomData,
        }
    }
}
