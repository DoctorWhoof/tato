use super::*;
use core::slice::Iter;

mod drain;
pub use drain::*;

#[derive(Debug, Clone)]
pub struct Buffer<T, Idx = u16, Marker = ()> {
    pub slice: Slice<T, Idx, Marker>,
    len: Idx, // Current number of elements used
}

impl<T, Idx, Marker> Buffer<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    pub fn new<const LEN: usize>(arena: &mut Arena<LEN, Idx, Marker>, capacity: Idx) -> Option<Self>
    where
        T: Default,
    {
        let slice = arena.alloc_slice::<T>(capacity)?;
        Some(Self { slice, len: Idx::zero() })
    }

    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, Idx, Marker>,
        capacity: Idx,
        func: F,
    ) -> Option<Self>
    where
        F: FnMut(usize) -> T,
    {
        let slice = arena.alloc_slice_from_fn(capacity, func)?;
        Some(Self { slice, len: capacity })
    }

    pub fn is_empty(&self) -> bool {
        self.len == Idx::zero()
    }

    pub fn clear(&mut self) {
        self.len = Idx::zero()
    }

    pub fn len(&self) -> usize {
        self.len.into()
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
    ) -> Result<(), &str> {
        if self.len >= self.slice.capacity() {
            return Err("Arena: Capacity reached"); // Return the value back if full
        }
        let slice = arena
            .get_slice_mut(&self.slice) //
            .expect("Arena: Can't push new item");
        slice[self.len.into()] = value;
        self.len += Idx::one();
        Ok(())
    }

    pub fn as_slice<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> Option<&'a [T]> {
        let full_slice = arena.get_slice(&self.slice)?;
        Some(&full_slice[..self.len.into()])
    }

    /// A Buffer of smaller buffers.
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena. "func" must return each individual sub-buffer.
    pub fn multi_buffer<const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, Idx, Marker>,
        sub_buffer_count: Idx,
        sub_buffer_len: Idx,
    ) -> Option<Buffer<Buffer<T, Idx, Marker>, Idx, Marker>>
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
        for i in 0..sub_buffer_count.into() {
            let buffer =
                Buffer::new(arena, sub_buffer_len).expect("Arena: Could not create buffer");
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
    ) -> Option<Iter<'a, T>> {
        arena.iter_slice_range(&self.slice, Idx::zero(), self.len)
    }

    pub fn drain<'a, const LEN: usize>(
        &'a mut self,
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> DrainIterator<'a, T, LEN, Idx, Marker> {
        let end = self.len.into();
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
