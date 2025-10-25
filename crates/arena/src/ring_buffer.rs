use super::*;
use crate::{ArenaErr, ArenaRes};

mod iter;
pub use iter::*;

/// Arena allocated FIFO Ring buffer
#[derive(Debug, Clone)]
pub struct RingBuffer<T, I = u32, M = ()> {
    pub slice: Slice<T, I, M>,
    head: I,  // Index of the first element
    len: I,   // Current number of elements used
}

impl<T, I, M> Default for RingBuffer<T, I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self {
            slice: Default::default(),
            head: Default::default(),
            len: Default::default(),
        }
    }
}

impl<T, I, M> RingBuffer<T, I, M>
where
    I: ArenaIndex,
{
    pub fn new<const LEN: usize>(
        arena: &mut Arena<LEN, I, M>,
        capacity: I,
    ) -> ArenaRes<Self> {
        let slice = arena.alloc_slice_uninit::<T>(capacity.to_usize())?;
        Ok(Self {
            slice,
            head: I::zero(),
            len: I::zero(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.len == I::zero()
    }

    pub fn is_full(&self) -> bool {
        self.len >= self.slice.capacity()
    }

    pub fn clear(&mut self) {
        self.head = I::zero();
        self.len = I::zero();
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

    /// Push a value to the back of the ring buffer (FIFO).
    /// Automatically overwrites the oldest element if buffer is full.
    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, I, M>,
        value: T,
    ) -> ArenaRes<()> {
        if self.slice.capacity() == I::zero() {
            return Err(ArenaErr::UnnallocatedObject);
        }
        let tail = self.tail_index();
        let slice = arena.get_slice_mut(&self.slice)?;
        slice[tail.to_usize()] = value;

        if self.is_full() {
            // Move head forward to overwrite oldest
            let capacity = self.slice.capacity().to_usize();
            self.head = I::from_usize_checked((self.head.to_usize() + 1) % capacity).unwrap();
        } else {
            self.len += I::one();
        }
        Ok(())
    }

    /// Push a value to the back of the ring buffer (FIFO).
    /// Returns error if buffer is full.
    pub fn try_push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, I, M>,
        value: T,
    ) -> ArenaRes<()> {
        if self.is_full() {
            return Err(ArenaErr::CapacityExceeded);
        }

        let tail = self.tail_index();
        let slice = arena.get_slice_mut(&self.slice)?;
        slice[tail.to_usize()] = value;
        self.len += I::one();
        Ok(())
    }

    /// Pop a value from the front of the ring buffer (FIFO).
    pub fn pop<const LEN: usize>(
        &mut self,
        arena: &Arena<LEN, I, M>,
    ) -> Option<T>
    where
        T: Copy,
    {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(&self.slice).expect("RingBuffer slice should always be valid");
        let value = slice[self.head.to_usize()];

        let capacity = self.slice.capacity().to_usize();
        self.head = I::from_usize_checked((self.head.to_usize() + 1) % capacity).unwrap();
        self.len -= I::one();
        Some(value)
    }

    /// Get a reference to the front element without removing it.
    pub fn front<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I, M>,
    ) -> Option<&'a T> {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(&self.slice).ok()?;
        Some(&slice[self.head.to_usize()])
    }

    /// Get a reference to the back element without removing it.
    pub fn back<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I, M>,
    ) -> Option<&'a T> {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(&self.slice).ok()?;
        let back_index = if self.len == I::one() {
            self.head
        } else {
            let capacity = self.slice.capacity().to_usize();
            let tail = self.tail_index().to_usize();
            let back_pos = (tail + capacity - 1) % capacity;
            I::from_usize_checked(back_pos).unwrap()
        };
        Some(&slice[back_index.to_usize()])
    }

    /// Get element at logical index (0 is front, 1 is next, etc).
    pub fn get<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I, M>,
        index: I,
    ) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }

        let slice = arena.get_slice(&self.slice).ok()?;
        let capacity = self.slice.capacity().to_usize();
        let physical_index = (self.head.to_usize() + index.to_usize()) % capacity;
        Some(&slice[physical_index])
    }

    /// Iterate over elements in logical order (from front to back).
    pub fn items<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, I, M>,
    ) -> RingBufferIterator<'a, T, LEN, I, M> {
        RingBufferIterator::new(arena, &self.slice, self.head, self.len)
    }


    fn tail_index(&self) -> I {
        let capacity = self.slice.capacity().to_usize();
        let tail = (self.head.to_usize() + self.len.to_usize()) % capacity;
        I::from_usize_checked(tail).unwrap()
    }
}
