use super::*;
use crate::{ArenaError, ArenaResult};
use core::slice::Iter;

mod drain;
pub use drain::*;

#[derive(Debug, Clone)]
pub struct Buffer<T, Idx = u16, Marker = ()> {
    pub slice: Slice<T, Idx, Marker>,
    len: Idx, // Current number of elements used
}

impl<T, Idx, Marker> Default for Buffer<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self { slice: Default::default(), len: Default::default() }
    }
}

impl<T, Idx, Marker> Buffer<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    pub fn new<const LEN: usize>(
        arena: &mut Arena<LEN, Idx, Marker>,
        capacity: Idx,
    ) -> ArenaResult<Self> {
        let slice = arena.alloc_slice_uninit::<T>(capacity)?;
        Ok(Self { slice, len: Idx::zero() })
    }

    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, Idx, Marker>,
        capacity: Idx,
        func: F,
    ) -> ArenaResult<Self>
    where
        F: FnMut(usize) -> T,
    {
        let slice = arena.alloc_slice_from_fn(capacity, func)?;
        Ok(Self { slice, len: capacity })
    }

    pub fn is_empty(&self) -> bool {
        self.len == Idx::zero()
    }

    pub fn clear(&mut self) {
        self.len = Idx::zero()
    }

    pub fn len(&self) -> usize {
        self.len.to_usize()
    }

    pub fn capacity(&self) -> Idx {
        self.slice.capacity()
    }

    pub fn remaining(&self) -> Idx {
        self.slice.capacity() - self.len
    }

    pub fn used(&self) -> Idx {
        self.len
    }

    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, Idx, Marker>,
        value: T,
    ) -> ArenaResult<()> {
        if self.len >= self.slice.capacity() {
            return Err(ArenaError::CapacityExceeded);
        }
        let slice = arena.get_slice_mut(&self.slice)?;
        slice[self.len.to_usize()] = value;
        self.len += Idx::one();
        Ok(())
    }

    pub fn truncate(&mut self, new_len: Idx) {
        if new_len >= self.len {
            return;
        }
        self.len = new_len;
    }

    /// Resizes the buffer within the capacity boundaries. If new length is longer
    /// than the current, the new items are filled with the default value.
    pub fn resize<const LEN: usize>(&mut self, arena: &mut Arena<LEN, Idx, Marker>, new_len: Idx)
    where
        T: Default,
    {
        if new_len >= self.slice.capacity() {
            return;
        }
        if new_len >= self.len {
            if let Ok(slice) = arena.get_slice_mut(&self.slice) {
                for item in slice {
                    *item = T::default()
                }
            }
        }
        self.len = new_len;
    }

    pub fn as_slice<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> ArenaResult<&'a [T]> {
        let full_slice = arena.get_slice(&self.slice)?;
        Ok(&full_slice[..self.len.to_usize()])
    }

    pub fn as_slice_mut<'a, const LEN: usize>(
        &self,
        arena: &'a mut Arena<LEN, Idx, Marker>,
    ) -> ArenaResult<&'a mut [T]> {
        let full_slice = arena.get_slice_mut(&self.slice)?;
        Ok(&mut full_slice[..self.len.to_usize()])
    }

    /// A Buffer of smaller buffers.
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena. "func" must return each individual sub-buffer.
    pub fn multi_buffer<const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, Idx, Marker>,
        sub_buffer_count: Idx,
        sub_buffer_len: Idx,
    ) -> ArenaResult<Buffer<Buffer<T, Idx, Marker>, Idx, Marker>>
    where
        T: Default,
    {
        // Allocate temporary space in the arena for the buffers using MaybeUninit
        let temp_slice = arena.alloc_slice_from_fn(sub_buffer_count, |_| {
            core::mem::MaybeUninit::<Buffer<T, Idx, Marker>>::uninit()
        })?;
        let temp_ptr =
            arena.get_slice_mut(&temp_slice)?.as_mut_ptr() as *mut Buffer<T, Idx, Marker>;

        // Initialize each buffer in the temporary space
        for i in 0..sub_buffer_count.to_usize() {
            let buffer = Buffer::new(arena, sub_buffer_len)?;
            unsafe {
                temp_ptr.add(i).write(buffer);
            }
        }

        // Create the final buffer by moving from temporary storage
        Buffer::from_fn(arena, sub_buffer_count, |i| unsafe { core::ptr::read(temp_ptr.add(i)) })
    }

    // Iterators
    pub fn items<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> ArenaResult<Iter<'a, T>> {
        arena.iter_slice_range(&self.slice, Idx::zero(), self.len)
    }

    pub fn drain<'a, const LEN: usize>(
        &'a mut self,
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> DrainIterator<'a, T, LEN, Idx, Marker> {
        let end = self.len.to_usize();
        let iter = DrainIterator {
            arena,
            slice: Slice::new(
                self.slice.offset(),
                self.slice.len(),
                self.slice.generation(),
                self.slice.arena_id(),
            ),
            current: 0,
            end,
        };
        self.len = Idx::zero();
        iter
    }
}




