use super::*;

#[derive(Debug, Clone)]
pub struct Buffer<T, Idx = u16> {
    pub pool: Slice<T, Idx>,
    len: Idx, // Current number of elements used
}

impl<T, Idx> Buffer<T, Idx>
where
    Idx: ArenaIndex,
{
    pub fn new<const LEN: usize>(arena: &mut Arena<LEN, Idx>, capacity: Idx) -> Option<Self>
    where
        T: Default,
    {
        let pool = arena.alloc_pool::<T>(capacity)?;
        Some(Self { pool, len: Idx::zero() })
    }

    pub fn from_fn<const LEN: usize, F>(
        arena: &mut Arena<LEN, Idx>,
        capacity: Idx,
        func: F,
    ) -> Option<Self>
    where
        F: FnMut(usize) -> T,
    {
        let pool = arena.alloc_pool_from_fn(capacity, func)?;
        Some(Self { pool, len: capacity })
    }

    pub fn clear(&mut self) {
        self.len = Idx::zero()
    }

    pub fn len(&self) -> usize {
        self.len.into()
    }

    pub fn capacity(&self) -> Idx {
        self.pool.capacity()
    }

    pub fn remaining(&self) -> Idx {
        self.pool.capacity() - self.len
    }

    pub fn used(&self) -> Idx {
        self.len
    }

    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, Idx>,
        value: T,
    ) -> Result<(), &str> {
        if self.len >= self.pool.capacity() {
            return Err("Arena: Capacity reached"); // Return the value back if full
        }
        let slice = arena
            .get_pool_mut(&self.pool) //
            .expect("Arena: Can't push new item");
        slice[self.len.into()] = value;
        self.len += Idx::one();
        Ok(())
    }

    pub fn as_slice<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, Idx>) -> Option<&'a [T]> {
        let full_slice = arena.get_pool(&self.pool)?;
        Some(&full_slice[..self.len.into()])
    }

    /// A Buffer of smaller buffers.
    /// Helps to get around borrowing issues since the buffer and the text lines
    /// are in the same arena. "func" must return each individual sub-buffer.
    pub fn multi_buffer<const ARENA_LEN: usize>(
        arena: &mut Arena<ARENA_LEN, Idx>,
        sub_buffer_count: Idx,
        sub_buffer_len: Idx,
    ) -> Option<Buffer<Buffer<T, Idx>, Idx>>
    where
        T: Default,
    {
        // Allocate temporary space in the arena for the buffers using MaybeUninit
        let temp_slice = arena.alloc_pool_from_fn(sub_buffer_count, |_| {
            core::mem::MaybeUninit::<Buffer<T, Idx>>::uninit()
        })?;
        let temp_ptr = arena.get_pool_mut(&temp_slice)?.as_mut_ptr() as *mut Buffer<T, Idx>;

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
}
