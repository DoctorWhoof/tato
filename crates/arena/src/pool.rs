//! Pools for arena allocations

use core::marker::PhantomData;
use tato_math::Num;

#[derive(Debug, Clone, Copy)]
pub struct Pool<T, SizeType = usize> {
    pub(crate) offset: SizeType,
    pub(crate) len: SizeType,
    pub(crate) _marker: PhantomData<T>,
}

impl<T, SizeType> Pool<T, SizeType> {
    /// Create a new pool (internal use)
    pub(crate) fn new(offset: SizeType, len: SizeType) -> Self {
        Self { offset, len, _marker: PhantomData }
    }

    /// Get element count
    pub fn len(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.len.into()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool
    where
        SizeType: Copy + Into<usize>,
    {
        self.len.into() == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.offset.into()
    }

    /// Get size in bytes
    pub fn size_bytes(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.len.into() * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> (usize, usize)
    where
        SizeType: Copy + Into<usize>,
    {
        (self.len.into(), self.len.into())
    }
}

impl<T, SizeType> Default for Pool<T, SizeType>
where
    SizeType: Num,
{
    fn default() -> Self {
        Self {
            offset: SizeType::zero(),
            len: SizeType::zero(),
            _marker: PhantomData,
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
}

impl core::fmt::Display for PoolError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PoolError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            PoolError::Empty => write!(f, "Pool is empty"),
            PoolError::NotInitialized => write!(f, "Pool is not initialized"),
        }
    }
}
