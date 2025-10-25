use super::*;

/// Lightweight temporary arena for per-frame allocations
pub struct TempArena<const LEN: usize, Idx = u32> {
    data: [u8; LEN],
    offset: usize,
    _marker: PhantomData<Idx>,
}

impl<const LEN: usize, Idx: ArenaIndex> TempArena<LEN, Idx> {
    /// Create a new temporary arena
    pub fn new() -> Self {
        Self { data: [0; LEN], offset: 0, _marker: PhantomData }
    }

    /// Allocate a single value
    pub fn alloc<T>(&mut self, value: T) -> TempArenaResult<TempID<T, Idx>> {
        let size = size_of::<T>();
        let align = align_of::<T>();

        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check if we have space
        if aligned_offset + size > LEN {
            return Err(TempArenaError::OutOfSpace {
                requested: size,
                available: LEN.saturating_sub(aligned_offset),
            });
        }

        // Write the value
        unsafe {
            let ptr = self.data.as_mut_ptr().add(aligned_offset) as *mut T;
            ptr::write(ptr, value);
        }

        let id = TempID::new(
            Idx::try_from(aligned_offset).map_err(|_| TempArenaError::IndexConversion)?,
            Idx::try_from(1).map_err(|_| TempArenaError::IndexConversion)?,
        );
        self.offset = aligned_offset + size;
        Ok(id)
    }

    /// Allocate a slice with default values
    pub fn alloc_slice<T>(&mut self, len: usize) -> TempArenaResult<TempID<[T], Idx>>
    where
        T: Default,
    {
        let size = size_of::<T>() * len;
        let align = align_of::<T>();

        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check if we have space
        if aligned_offset + size > LEN {
            return Err(TempArenaError::OutOfSpace {
                requested: size,
                available: LEN.saturating_sub(aligned_offset),
            });
        }

        // Initialize with default values
        unsafe {
            let ptr = self.data.as_mut_ptr().add(aligned_offset) as *mut T;
            for i in 0..len {
                ptr::write(ptr.add(i), T::default());
            }
        }

        let id = TempID::new(
            Idx::try_from(aligned_offset).map_err(|_| TempArenaError::IndexConversion)?,
            Idx::try_from(len).map_err(|_| TempArenaError::IndexConversion)?,
        );
        self.offset = aligned_offset + size;
        Ok(id)
    }

    /// Allocate a slice from an iterator
    pub fn alloc_slice_from_iter<T, I>(&mut self, iter: I) -> TempArenaResult<TempID<[T], Idx>>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let len = iter.len();

        let size = size_of::<T>() * len;
        let align = align_of::<T>();

        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check if we have space
        if aligned_offset + size > LEN {
            return Err(TempArenaError::OutOfSpace {
                requested: size,
                available: LEN.saturating_sub(aligned_offset),
            });
        }

        // Initialize with iterator values
        unsafe {
            let ptr = self.data.as_mut_ptr().add(aligned_offset) as *mut T;
            for (i, value) in iter.enumerate() {
                ptr::write(ptr.add(i), value);
            }
        }

        let id = TempID::new(
            Idx::try_from(aligned_offset).map_err(|_| TempArenaError::IndexConversion)?,
            Idx::try_from(len).map_err(|_| TempArenaError::IndexConversion)?,
        );
        self.offset = aligned_offset + size;
        Ok(id)
    }

    /// Allocate a slice from a function
    pub fn alloc_slice_from_fn<T, F>(
        &mut self,
        len: usize,
        f: F,
    ) -> TempArenaResult<TempID<[T], Idx>>
    where
        F: Fn(usize) -> T,
    {
        let size = size_of::<T>() * len;
        let align = align_of::<T>();

        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);

        // Check if we have space
        if aligned_offset + size > LEN {
            return Err(TempArenaError::OutOfSpace {
                requested: size,
                available: LEN.saturating_sub(aligned_offset),
            });
        }

        // Initialize with function
        unsafe {
            let ptr = self.data.as_mut_ptr().add(aligned_offset) as *mut T;
            for i in 0..len {
                ptr::write(ptr.add(i), f(i));
            }
        }

        let id = TempID::new(
            Idx::try_from(aligned_offset).map_err(|_| TempArenaError::IndexConversion)?,
            Idx::try_from(len).map_err(|_| TempArenaError::IndexConversion)?,
        );
        self.offset = aligned_offset + size;
        Ok(id)
    }

    /// Get reference to allocated value
    pub fn get<T>(&self, id: TempID<T, Idx>) -> Option<&T> {
        if id.offset() + size_of::<T>() > LEN {
            return None;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(id.offset()) as *const T;
            Some(&*ptr)
        }
    }

    /// Get mutable reference to allocated value
    pub fn get_mut<T>(&mut self, id: TempID<T, Idx>) -> Option<&mut T> {
        if id.offset() + size_of::<T>() > LEN {
            return None;
        }

        unsafe {
            let ptr = self.data.as_mut_ptr().add(id.offset()) as *mut T;
            Some(&mut *ptr)
        }
    }

    /// Get reference to allocated slice
    pub fn get_slice<T>(&self, id: TempID<[T], Idx>) -> Option<&[T]> {
        let size = size_of::<T>() * id.len();
        if id.offset() + size > LEN {
            return None;
        }

        unsafe {
            let ptr = self.data.as_ptr().add(id.offset()) as *const T;
            Some(slice::from_raw_parts(ptr, id.len()))
        }
    }

    /// Get mutable reference to allocated slice (assumes initialized)
    pub fn get_slice_mut<T>(&mut self, id: TempID<[T], Idx>) -> Option<&mut [T]> {
        let size = size_of::<T>() * id.len();
        if id.offset() + size > LEN {
            return None;
        }

        unsafe {
            let ptr = self.data.as_mut_ptr().add(id.offset()) as *mut T;
            Some(slice::from_raw_parts_mut(ptr, id.len()))
        }
    }



    /// Get current used bytes
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Get remaining bytes
    pub fn remaining(&self) -> usize {
        LEN - self.offset
    }

    /// Get total capacity
    pub fn capacity(&self) -> usize {
        LEN
    }
}

impl<const LEN: usize, Idx: ArenaIndex> Default for TempArena<LEN, Idx> {
    fn default() -> Self {
        Self::new()
    }
}