use super::*;
use crate::{ArenaError, ArenaResult};

#[derive(Debug, Clone)]
pub struct RingBuffer<T, Idx = u32, Marker = ()> {
    pub slice: Slice<T, Idx, Marker>,
    head: Idx,  // Index of the first element
    len: Idx,   // Current number of elements used
}

impl<T, Idx, Marker> Default for RingBuffer<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self { 
            slice: Default::default(), 
            head: Default::default(),
            len: Default::default(),
        }
    }
}

impl<T, Idx, Marker> RingBuffer<T, Idx, Marker>
where
    Idx: ArenaIndex,
{
    pub fn new<const LEN: usize>(
        arena: &mut Arena<LEN, Idx, Marker>,
        capacity: Idx,
    ) -> ArenaResult<Self> {
        let slice = arena.alloc_slice_uninit::<T>(capacity)?;
        Ok(Self { 
            slice, 
            head: Idx::zero(),
            len: Idx::zero(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.len == Idx::zero()
    }

    pub fn is_full(&self) -> bool {
        self.len >= self.slice.capacity()
    }

    pub fn clear(&mut self) {
        self.head = Idx::zero();
        self.len = Idx::zero();
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

    fn tail_index(&self) -> Idx {
        let capacity = self.slice.capacity().to_usize();
        let tail = (self.head.to_usize() + self.len.to_usize()) % capacity;
        Idx::from_usize_checked(tail).unwrap()
    }

    /// Push a value to the back of the ring buffer (FIFO).
    /// Returns error if buffer is full.
    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, Idx, Marker>,
        value: T,
    ) -> ArenaResult<()> {
        if self.is_full() {
            return Err(ArenaError::CapacityExceeded);
        }

        let tail = self.tail_index();
        let slice = arena.get_slice_mut(&self.slice)?;
        slice[tail.to_usize()] = value;
        self.len += Idx::one();
        Ok(())
    }

    /// Push a value to the back, overwriting the oldest element if full.
    pub fn push_overwrite<const LEN: usize>(
        &mut self,
        arena: &mut Arena<LEN, Idx, Marker>,
        value: T,
    ) -> ArenaResult<()> {
        let tail = self.tail_index();
        let slice = arena.get_slice_mut(&self.slice)?;
        slice[tail.to_usize()] = value;

        if self.is_full() {
            // Move head forward to overwrite oldest
            let capacity = self.slice.capacity().to_usize();
            self.head = Idx::from_usize_checked((self.head.to_usize() + 1) % capacity).unwrap();
        } else {
            self.len += Idx::one();
        }
        Ok(())
    }

    /// Pop a value from the front of the ring buffer (FIFO).
    pub fn pop<const LEN: usize>(
        &mut self,
        arena: &Arena<LEN, Idx, Marker>,
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
        self.head = Idx::from_usize_checked((self.head.to_usize() + 1) % capacity).unwrap();
        self.len -= Idx::one();
        Some(value)
    }

    /// Get a reference to the front element without removing it.
    pub fn front<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, Idx, Marker>,
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
        arena: &'a Arena<LEN, Idx, Marker>,
    ) -> Option<&'a T> {
        if self.is_empty() {
            return None;
        }

        let slice = arena.get_slice(&self.slice).ok()?;
        let back_index = if self.len == Idx::one() {
            self.head
        } else {
            let capacity = self.slice.capacity().to_usize();
            let tail = self.tail_index().to_usize();
            let back_pos = (tail + capacity - 1) % capacity;
            Idx::from_usize_checked(back_pos).unwrap()
        };
        Some(&slice[back_index.to_usize()])
    }

    /// Get element at logical index (0 is front, 1 is next, etc).
    pub fn get<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<LEN, Idx, Marker>,
        index: Idx,
    ) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }

        let slice = arena.get_slice(&self.slice).ok()?;
        let capacity = self.slice.capacity().to_usize();
        let physical_index = (self.head.to_usize() + index.to_usize()) % capacity;
        Some(&slice[physical_index])
    }
}