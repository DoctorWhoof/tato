use super::*;
use core::marker::PhantomData;

/// Handle to a value allocated in a buffer
#[derive(Debug, PartialEq, Eq)]
pub struct TempID<T: ?Sized, Idx = u32> {
    /// Byte offset in the buffer
    offset: Idx,
    /// Number of items (1 for single values, N for slices)
    len: Idx,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, Idx: Copy> Copy for TempID<T, Idx> {}

impl<T: ?Sized, Idx: Copy> Clone for TempID<T, Idx> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, Idx: ArenaIndex> TempID<T, Idx> {
    pub(super) fn new(offset: Idx, len: Idx) -> Self {
        Self { offset, len, _phantom: PhantomData }
    }

    pub fn offset(&self) -> usize {
        self.offset.to_usize()
    }

    pub fn len(&self) -> usize {
        self.len.to_usize()
    }


}
