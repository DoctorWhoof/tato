use super::*;

#[derive(Debug, Clone)]
pub struct Buffer<T, Idx = u16> {
    pub pool: Slice<T, Idx>,
    len: Idx,      // Current number of elements used
    capacity: Idx, // Maximum elements (from original allocation)
}

impl<T, Idx: ArenaIndex> Buffer<T, Idx> {
    pub fn new<const LEN: usize>(arena: &mut Arena<LEN, Idx>, capacity: Idx) -> Option<Self>
    where
        T: Default,
    {
        let pool = arena.alloc_pool::<T>(capacity)?;
        Some(Self { pool, len: Idx::zero(), capacity })
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
        Some(Self { pool, len:capacity, capacity })
    }

    pub fn clear(&mut self) {
        self.len = Idx::zero()
    }

    pub fn len(&self) -> usize {
        self.len.into()
    }

    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }

    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, Idx>,
        value: T,
    ) -> Result<(), &str> {
        if self.len >= self.capacity {
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
    pub fn multi_buffer<const LEN: usize, const ARENA_LEN: usize, F>(
        arena: &mut Arena<ARENA_LEN, Idx>,
        sub_buffer_count: Idx,
        sub_buffer_len: Idx,
        item_func: F,
    ) -> Option<Buffer<Buffer<T, Idx>, Idx>>
    where
        T: Clone,
        F: FnMut(usize) -> T + Copy,
    {
        let buffers: [Buffer<T, Idx>; LEN] = core::array::from_fn(|_| {
            Buffer::from_fn(arena, sub_buffer_len, item_func)
                .expect("Arena: Could not create buffer")
        });

        // Return buffer, moving items from the array
        Buffer::from_fn(arena, sub_buffer_count, |i| buffers[i].clone())
    }
}
