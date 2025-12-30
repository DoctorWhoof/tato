use super::*;
use crate::{ArenaErr, ArenaOps, ArenaRes};
use core::slice::Iter;

mod drain;
pub use drain::*;

#[derive(Debug, Clone)]
pub struct Buffer<T, I = u32, M = ()> {
    pub slice: Slice<T, I, M>,
    len: I, // Current number of elements used
}

impl<T, I, M> Default for Buffer<T, I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self { slice: Default::default(), len: Default::default() }
    }
}

impl<T, I, M> Buffer<T, I, M>
where
    I: ArenaIndex,
{
    pub fn new<const LEN: usize>(
        arena: &mut Arena<LEN, I, M>,
        capacity: I,
    ) -> ArenaRes<Self> {
        let (slice, _) = arena.alloc_slice_uninit::<T>(capacity.to_usize())?;
        Ok(Self { slice, len: I::zero() })
    }

    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, I, M>,
        capacity: I,
        func: F,
    ) -> ArenaRes<Self>
    where
        F: Fn(usize) -> T,
    {
        let slice = arena.alloc_slice_from_fn(capacity.to_usize(), func)?;
        Ok(Self { slice, len: capacity })
    }

    pub fn is_empty(&self) -> bool {
        self.len == I::zero()
    }

    pub fn clear(&mut self) {
        self.len = I::zero()
    }

    pub fn len(&self) -> usize {
        self.len.to_usize()
    }

    pub fn capacity(&self) -> I {
        self.slice.capacity()
    }

    pub fn remaining(&self) -> I {
        self.slice.capacity() - self.len
    }

    pub fn used(&self) -> I {
        self.len
    }

    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, I, M>,
        value: T,
    ) -> ArenaRes<()> {
        if self.slice.capacity() == I::zero() {
            return Err(ArenaErr::UnnallocatedObject);
        }
        if self.len >= self.slice.capacity() {
            return Err(ArenaErr::CapacityExceeded);
        }
        let slice = arena.get_slice_mut(self.slice.clone())?;
        slice[self.len.to_usize()] = value;
        self.len += I::one();
        Ok(())
    }

    pub fn pop<const LEN: usize>(
        &mut self,
        arena: &Arena<LEN, I, M>,
    ) -> Option<T>
    where
        T: Copy,
    {
        if self.len == I::zero() {
            return None;
        }

        self.len -= I::one();
        let slice = arena.get_slice(self.slice.clone()).expect("Buffer slice should always be valid");
        Some(slice[self.len.to_usize()])
    }

    pub fn truncate(&mut self, new_len: I) {
        if new_len >= self.len {
            return;
        }
        self.len = new_len;
    }

    /// Resizes the buffer within the capacity boundaries. If new length is longer
    /// than the current, the new items are filled with the default value.
    pub fn resize<const LEN: usize>(&mut self, arena: &mut Arena<LEN, I, M>, new_len: I)
    where
        T: Default,
    {
        if new_len >= self.slice.capacity() {
            return;
        }
        if new_len >= self.len {
            if let Ok(slice) = arena.get_slice_mut(self.slice.clone()) {
                for item in slice {
                    *item = T::default()
                }
            }
        }
        self.len = new_len;
    }

    pub fn as_slice<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I, M>,
    ) -> ArenaRes<&'a [T]> {
        let full_slice = arena.get_slice(self.slice.clone())?;
        Ok(&full_slice[..self.len.to_usize()])
    }

    pub fn as_slice_mut<'a, const LEN: usize>(
        &self,
        arena: &'a mut Arena<LEN, I, M>,
    ) -> ArenaRes<&'a mut [T]> {
        let full_slice = arena.get_slice_mut(self.slice.clone())?;
        Ok(&mut full_slice[..self.len.to_usize()])
    }

    /// A Buffer of smaller buffers.
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena. "func" must return each individual sub-buffer.
    pub fn multi_buffer<const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, I, M>,
        sub_buffer_count: I,
        sub_buffer_len: I,
    ) -> ArenaRes<Buffer<Buffer<T, I, M>, I, M>>
    where
        T: Default,
    {
        // Allocate temporary space in the arena for the buffers using MaybeUninit
        let temp_slice = arena.alloc_slice_from_fn(sub_buffer_count.to_usize(), |_| {
            core::mem::MaybeUninit::<Buffer<T, I, M>>::uninit()
        })?;
        let temp_ptr =
            arena.get_slice_mut(temp_slice)?.as_mut_ptr() as *mut Buffer<T, I, M>;

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
        arena: &'a Arena<LEN, I, M>,
    ) -> ArenaRes<Iter<'a, T>> {
        arena.iter_slice_range(self.slice.clone(), 0, self.len.to_usize())
    }

    pub fn drain<'a, const LEN: usize>(
        &'a mut self,
        arena: &'a Arena<LEN, I, M>,
    ) -> DrainIterator<'a, T, LEN, I, M> {
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
        self.len = I::zero();
        iter
    }
}
