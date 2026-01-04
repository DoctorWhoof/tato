use super::*;
use crate::{ArenaErr, ArenaOps, ArenaRes};

mod iter;
pub use iter::*;

/// Arena allocated FIFO Ring buffer
#[derive(Debug, Clone)]
pub struct RingBuffer<T, I = u32, M = ()> {
    pub slice: Slice<T, I, M>,
    head: I, // Index of the first element
    len: I,  // Current number of elements used
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
    pub fn new<A>(arena: &mut A, capacity: I) -> ArenaRes<Self>
    where
        A: ArenaOps<I, M>,
    {
        let (slice, _) = arena.alloc_slice_uninit::<T>(capacity.to_usize())?;
        Ok(Self { slice, head: I::zero(), len: I::zero() })
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
    pub fn push<A>(&mut self, arena: &mut A, value: T) -> ArenaRes<()>
    where
        A: ArenaOps<I, M>,
    {
        if self.slice.capacity() == I::zero() {
            return Err(ArenaErr::UnnallocatedObject);
        }
        let tail = self.tail_index();
        let slice = arena.get_slice_mut(self.slice.clone())?;
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
    pub fn try_push<A>(&mut self, arena: &mut A, value: T) -> ArenaRes<()>
    where
        A: ArenaOps<I, M>,
    {
        if self.is_full() {
            return Err(ArenaErr::CapacityExceeded);
        }

        let tail = self.tail_index();
        let slice = arena.get_slice_mut(self.slice.clone())?;
        slice[tail.to_usize()] = value;
        self.len += I::one();
        Ok(())
    }

    /// Pop a value from the front of the ring buffer (FIFO).
    pub fn pop<A>(&mut self, arena: &A) -> Option<T>
    where
        T: Copy,
        A: ArenaOps<I, M>,
    {
        if self.is_empty() {
            return None;
        }

        let slice =
            arena.get_slice(self.slice.clone()).expect("RingBuffer slice should always be valid");
        let value = slice[self.head.to_usize()];

        let capacity = self.slice.capacity().to_usize();
        self.head = I::from_usize_checked((self.head.to_usize() + 1) % capacity).unwrap();
        self.len -= I::one();
        Some(value)
    }

    /// Get a reference to the front element without removing it.
    pub fn front<'a, A>(&self, arena: &'a A) -> Option<&'a T>
    where
        A: ArenaOps<I, M>,
    {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(self.slice.clone()).ok()?;
        Some(&slice[self.head.to_usize()])
    }

    /// Get a reference to the back element without removing it.
    pub fn back<'a, A>(&self, arena: &'a A) -> Option<&'a T>
    where
        A: ArenaOps<I, M>,
    {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(self.slice.clone()).ok()?;
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
    pub fn get<'a, A>(&self, arena: &'a A, index: I) -> Option<&'a T>
    where
        A: ArenaOps<I, M>,
    {
        if index >= self.len {
            return None;
        }

        let slice = arena.get_slice(self.slice.clone()).ok()?;
        let capacity = self.slice.capacity().to_usize();
        let physical_index = (self.head.to_usize() + index.to_usize()) % capacity;
        Some(&slice[physical_index])
    }

    /// Iterate over elements in logical order (from front to back).
    pub fn items<'a, A>(
        &self,
        arena: &'a A,
    ) -> RingBufferIterator<'a, A, T, I, M>
    where
        A: ArenaOps<I, M>,
    {
        RingBufferIterator::new(arena, &self.slice, self.head, self.len)
    }

    fn tail_index(&self) -> I {
        let capacity = self.slice.capacity().to_usize();
        let tail = (self.head.to_usize() + self.len.to_usize()) % capacity;
        I::from_usize_checked(tail).unwrap()
    }
}
