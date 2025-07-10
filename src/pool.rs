//! Pool - Fixed-size collections

use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct Pool<T> {
    pub(crate) offset: usize,
    pub(crate) len: usize,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> Pool<T> {
    /// Create a new pool (internal use)
    pub(crate) fn new(offset: usize, len: usize) -> Self {
        Self {
            offset,
            len,
            _marker: PhantomData,
        }
    }

    /// Get element count
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get arena offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get size in bytes
    pub fn size_bytes(&self) -> usize {
        self.len * core::mem::size_of::<T>()
    }

    /// Get capacity as (used, total)
    pub fn capacity(&self) -> (usize, usize) {
        (self.len, self.len)
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
