//! Pools for arena allocations

use core::marker::PhantomData;
use crate::ArenaIndex;

#[derive(Debug, Clone, Copy)]
pub struct Pool<T, SizeType = usize, Marker = ()> {
    pub(crate) offset: SizeType,
    pub(crate) len: SizeType,
    pub(crate) generation: u32,
    pub(crate) arena_id: u32,
    pub(crate) _phantom: PhantomData<(T, Marker)>,
}

impl<T, SizeType, Marker> Pool<T, SizeType, Marker> {
    /// Create a new pool (internal use)
    pub(crate) fn new(offset: SizeType, len: SizeType, generation: u32, arena_id: u32) -> Self {
        Self { 
            offset, 
            len, 
            generation,
            arena_id,
            _phantom: PhantomData 
        }
    }

    /// Get element count
    pub fn len(&self) -> usize
    where
        SizeType: ArenaIndex,
    {
        self.len.into()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool
    where
        SizeType: ArenaIndex,
    {
        self.len.into() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> usize
    where
        SizeType: ArenaIndex,
    {
        self.offset.into()
    }

    /// Get generation
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u32 {
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

/// Pool errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    /// Index out of bounds
    IndexOutOfBounds,
    /// Pool is empty
    Empty,
    /// Pool not initialized
    NotInitialized,
    /// Stale handle (generation mismatch)
    StaleHandle,
}

impl core::fmt::Display for PoolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PoolError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            PoolError::Empty => write!(f, "Pool is empty"),
            PoolError::NotInitialized => write!(f, "Pool is not initialized"),
            PoolError::StaleHandle => write!(f, "Stale handle (generation mismatch)"),
        }
    }
}