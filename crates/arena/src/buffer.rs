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
    pub fn new<A>(arena: &mut A, capacity: I) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
        let (slice, _) = arena.alloc_slice_uninit::<T>(capacity.to_usize())?;
        Ok(Self { slice, len: I::zero() })
    }

    pub fn from_fn<A, F>(arena: &mut A, capacity: I, func: F) -> ArenaRes<Self>
    where
        F: Fn(usize) -> T,
        A: ArenaOps<I, M>,
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

    pub fn push<A>(&mut self, arena: &mut A, value: T) -> ArenaRes<()>
    where
        A: ArenaOps<I, M>,
    {
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

    pub fn pop<A>(&mut self, arena: &mut A) -> Option<T>
    where
        T: Copy,
        A: ArenaOps<I, M>,
    {
        if self.len == I::zero() {
            return None;
        }

        self.len -= I::one();
        let slice =
            arena.get_slice(self.slice.clone()).expect("Buffer slice should always be valid");
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
    pub fn resize<A>(&mut self, arena: &mut A, new_len: I)
    where
        T: Default,
        A: ArenaOps<I, M>,
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

    pub fn get<A>(&self, arena: &A, index: usize) -> Option<T>
    where
        T: Copy,
        A: ArenaOps<I, M>,
    {
        if index >= self.len.to_usize() {
            return None;
        }

        let slice = arena.get_slice(self.slice.clone()).ok()?;
        Some(slice[index])
    }

    pub fn as_slice<'a, A>(&self, arena: &'a A) -> ArenaRes<&'a [T]>
    where
        A: ArenaOps<I, M>,
    {
        let full_slice = arena.get_slice(self.slice.clone())?;
        Ok(&full_slice[..self.len.to_usize()])
    }

    pub fn as_slice_mut<'a, A>(&self, arena: &'a mut A) -> ArenaRes<&'a mut [T]>
    where
        A: ArenaOps<I, M>,
    {
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
        let temp_ptr = arena.get_slice_mut(temp_slice)?.as_mut_ptr() as *mut Buffer<T, I, M>;

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
    pub fn items<'a, A>(&self, arena: &'a A) -> ArenaRes<Iter<'a, T>>
    where A: ArenaOps<I, M>,
    {
        arena.iter_slice_range(self.slice.clone(), 0, self.len.to_usize())
    }

    pub fn drain<'a, A>(&'a mut self, arena: &'a mut A) -> DrainIterator<'a, T, A, I, M>
    where A: ArenaOps<I, M>,
    {
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
