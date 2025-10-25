use super::*;

/// A growable buffer with fixed capacity allocated from a TempArena
/// 
/// This buffer pre-allocates space filled with default values, but only tracks
/// how many items have been "pushed" (are considered active). Popping simply 
/// decrements the counter without destroying data, allowing for efficient reuse.
/// 
/// All buffer operations work with fully initialized data - there are no 
/// uninitialized memory concerns.
/// 
/// Note: TempBuffer does not call Drop on items when popped or when the buffer
/// is dropped. This is by design for performance. Only use with types that don't
/// require cleanup (primitives, Copy types, etc.).
#[derive(Debug, Clone, Copy)]
pub struct TempBuffer<T, Idx: Copy = u32> {
    slice_id: TempID<[T], Idx>,
    len: usize,
}

impl<T, Idx: ArenaIndex + Copy> TempBuffer<T, Idx> {
    /// Create a buffer with the specified capacity
    /// 
    /// The buffer is pre-allocated and filled with `T::default()` values, 
    /// but starts with length 0 (no "active" items).
    pub fn with_capacity<const LEN: usize>(
        arena: &mut TempArena<LEN, Idx>,
        capacity: usize,
    ) -> TempArenaResult<Self>
    where
        T: Default,
    {
        let slice_id = arena.alloc_slice::<T>(capacity)?;
        Ok(Self { slice_id, len: 0 })
    }
    
    /// Create a buffer from an iterator (capacity = iterator length)
    pub fn from_iter<const LEN: usize, I>(
        arena: &mut TempArena<LEN, Idx>,
        iter: I,
    ) -> TempArenaResult<Self>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let capacity = iter.len();
        let slice_id = arena.alloc_slice_from_iter(iter)?;
        Ok(Self { slice_id, len: capacity })
    }

    /// Create a buffer from a function (capacity = len)
    pub fn from_fn<const LEN: usize, F>(
        arena: &mut TempArena<LEN, Idx>,
        len: usize,
        f: F,
    ) -> TempArenaResult<Self>
    where
        F: Fn(usize) -> T,
    {
        let slice_id = arena.alloc_slice_from_fn(len, f)?;
        Ok(Self { slice_id, len })
    }

    /// Push an item to the end of the buffer
    pub fn push<const LEN: usize>(
        &mut self,
        arena: &mut TempArena<LEN, Idx>,
        item: T,
    ) -> TempArenaResult<()> {
        if self.len >= self.capacity() {
            return Err(TempArenaError::BufferFull);
        }
        
        let slice = arena.get_slice_mut(self.slice_id).ok_or(TempArenaError::InvalidBounds)?;
        slice[self.len] = item;
        self.len += 1;
        
        Ok(())
    }

    /// Pop an item from the end of the buffer
    /// 
    /// This only decrements the counter - the item remains in memory until overwritten
    pub fn pop<const LEN: usize>(
        &mut self,
        arena: &TempArena<LEN, Idx>,
    ) -> Option<T> 
    where
        T: Copy,
    {
        if self.len == 0 {
            return None;
        }
        
        let slice = arena.get_slice(self.slice_id)?;
        self.len -= 1;
        Some(slice[self.len])
    }

    /// Clear the buffer (reset length to 0)
    /// 
    /// Items remain in memory until overwritten
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Get current length (number of pushed items)
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get total capacity (maximum items)
    pub fn capacity(&self) -> usize {
        self.slice_id.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if buffer is at full capacity
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity()
    }

    /// Get remaining space
    pub fn remaining(&self) -> usize {
        self.capacity() - self.len
    }

    /// Get as slice of initialized items only
    pub fn as_slice<'a, const LEN: usize>(&self, arena: &'a TempArena<LEN, Idx>) -> Option<&'a [T]> {
        let slice = arena.get_slice(self.slice_id)?;
        Some(&slice[..self.len])
    }

    /// Get as mutable slice of initialized items only  
    pub fn as_slice_mut<'a, const LEN: usize>(&mut self, arena: &'a mut TempArena<LEN, Idx>) -> Option<&'a mut [T]> {
        let slice = arena.get_slice_mut(self.slice_id)?;
        Some(&mut slice[..self.len])
    }

    /// Get element at index (bounds checked against length)
    pub fn get<'a, const LEN: usize>(&self, arena: &'a TempArena<LEN, Idx>, index: usize) -> Option<&'a T> {
        if index >= self.len {
            return None;
        }
        
        self.as_slice(arena)?.get(index)
    }

    /// Get mutable element at index (bounds checked against length)
    pub fn get_mut<'a, const LEN: usize>(&mut self, arena: &'a mut TempArena<LEN, Idx>, index: usize) -> Option<&'a mut T> {
        if index >= self.len {
            return None;
        }
        
        self.as_slice_mut(arena)?.get_mut(index)
    }
}