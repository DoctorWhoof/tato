use core::marker::PhantomData;
use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;
use core::slice::Iter;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::{ArenaErr, ArenaId, ArenaIndex, ArenaRes, Slice};

// Global counter for unique arena IDs (no-std compatible)
static ARENA_ID_COUNTER: AtomicU16 = AtomicU16::new(1);

#[derive(Debug, Clone, Copy)]
struct RawAllocId<I> {
    offset: I,
    size: I,
}

/// Fixed-size arena with generational safety.
/// LEN = bytes, I = handle size, M = type safety marker.
#[repr(C, align(16))]
#[derive(Debug)]
pub struct Arena<const LEN: usize, I = u32, M = ()> {
    /// Raw storage for all allocations
    storage: [MaybeUninit<u8>; LEN],
    /// Current allocation offset (bump pointer)
    offset: I,
    /// Current tail allocation offset (allocates backwards from end)
    tail_offset: I,
    /// Current generation - incremented on restore_to()
    generation: u32,
    /// Unique arena ID for cross-arena safety
    arena_id: u16,
    /// Last allocation for pop() support
    last_alloc: Option<RawAllocId<I>>,
    /// Zero-sized type marker for compile-time arena safety
    _marker: PhantomData<M>,
}

impl<const LEN: usize, I, M> Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    /// Create a new arena with automatic cross-arena safety.
    /// Each arena gets a unique ID from an atomic counter, ensuring
    /// collision resistance and automatic cross-arena safety without requiring
    /// explicit marker types.
    pub fn new() -> Self {
        // Use atomic counter for guaranteed uniqueness
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: I::try_from(0).unwrap_or_else(|_| panic!("I too small")),
            tail_offset: I::try_from(LEN).unwrap_or_else(|_| panic!("I too small for LEN")),
            generation: 0,
            arena_id: ARENA_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            last_alloc: None,
            _marker: PhantomData,
        }
    }

    /// Allocate and store a value
    pub fn alloc<T>(&mut self, value: T) -> ArenaRes<ArenaId<T, I, M>>
    where
        T: 'static,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let offset_usize: usize = self.offset.to_usize();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset =
            if misalignment != 0 { offset_usize + align - misalignment } else { offset_usize };

        // Check space
        if aligned_offset + size > LEN {
            return Err(ArenaErr::OutOfSpace { requested: size, available: LEN - aligned_offset });
        }

        self.offset = I::try_from(aligned_offset).map_err(|_| ArenaErr::IndexConversion)?;

        // Store value
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            ptr::write(dst, value);
        }

        let id = ArenaId::new(
            self.offset,
            I::try_from(size).map_err(|_| ArenaErr::IndexConversion)?,
            self.generation,
            self.arena_id,
        );

        // Track this allocation for pop()
        self.last_alloc = Some(RawAllocId {
            offset: self.offset,
            size: I::try_from(size).map_err(|_| ArenaErr::IndexConversion)?,
        });

        self.offset = self.offset + I::try_from(size).map_err(|_| ArenaErr::IndexConversion)?;

        Ok(id)
    }

    /// Allocate slice with initialization function
    pub fn alloc_slice_from_fn<T, F>(&mut self, len: usize, mut f: F) -> ArenaRes<Slice<T, I, M>>
    where
        F: FnMut(usize) -> T,
    {
        if len == 0 {
            return Ok(Slice::new(
                self.offset,
                I::try_from(0).map_err(|_| ArenaErr::IndexConversion)?,
                self.generation,
                self.arena_id,
            ));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * len;
        let offset_usize: usize = self.offset.to_usize();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset =
            if misalignment != 0 { offset_usize + align - misalignment } else { offset_usize };

        // Check space
        if aligned_offset + total_size > LEN {
            return Err(ArenaErr::OutOfSpace {
                requested: total_size,
                available: LEN - aligned_offset,
            });
        }

        self.offset = I::try_from(aligned_offset).map_err(|_| ArenaErr::IndexConversion)?;

        // Initialize elements
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            for i in 0..len {
                ptr::write(dst.add(i), f(i));
            }
        }

        let slice = Slice::new(
            self.offset,
            I::try_from(len).map_err(|_| ArenaErr::IndexConversion)?,
            self.generation,
            self.arena_id,
        );

        // Track this allocation for pop()
        self.last_alloc = Some(RawAllocId {
            offset: self.offset,
            size: I::try_from(total_size).map_err(|_| ArenaErr::IndexConversion)?,
        });

        self.offset =
            self.offset + I::try_from(total_size).map_err(|_| ArenaErr::IndexConversion)?;

        Ok(slice)
    }

    /// Allocate slice with default values
    pub fn alloc_slice<T>(&mut self, len: usize) -> ArenaRes<Slice<T, I, M>>
    where
        T: Default,
    {
        self.alloc_slice_from_fn(len, |_| T::default())
    }

    /// Allocate slice from iterator
    pub fn alloc_slice_from_iter<T>(&mut self, iter: I) -> ArenaRes<Slice<T, I, M>>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        let len = iter.len();

        self.alloc_slice_from_fn(len, |_| iter.next().expect("ExactSizeIterator length mismatch"))
    }

    /// Allocate uninitialized slice
    pub fn alloc_slice_uninit<T>(&mut self, len: usize) -> ArenaRes<Slice<T, I, M>> {
        if len == 0 {
            return Ok(Slice::new(
                self.offset,
                I::try_from(0).map_err(|_| ArenaErr::IndexConversion)?,
                self.generation,
                self.arena_id,
            ));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * len;
        let offset_usize: usize = self.offset.to_usize();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset =
            if misalignment != 0 { offset_usize + align - misalignment } else { offset_usize };

        // Check space
        if aligned_offset + total_size > LEN {
            return Err(ArenaErr::OutOfSpace {
                requested: total_size,
                available: LEN - aligned_offset,
            });
        }

        self.offset = I::try_from(aligned_offset).map_err(|_| ArenaErr::IndexConversion)?;

        let slice = Slice::new(
            self.offset,
            I::try_from(len).map_err(|_| ArenaErr::IndexConversion)?,
            self.generation,
            self.arena_id,
        );

        // Track this allocation for pop()
        self.last_alloc = Some(RawAllocId {
            offset: self.offset,
            size: I::try_from(total_size).map_err(|_| ArenaErr::IndexConversion)?,
        });

        self.offset =
            self.offset + I::try_from(total_size).map_err(|_| ArenaErr::IndexConversion)?;

        Ok(slice)
    }

    /// Validate an ArenaId for safe access
    #[inline]
    fn validate_id<T>(&self, id: &ArenaId<T, I, M>) -> ArenaRes<()> {
        // Check arena ID first (cross-arena safety)
        if id.arena_id != self.arena_id {
            return Err(ArenaErr::CrossArenaAccess {
                expected_id: self.arena_id,
                found_id: id.arena_id,
            });
        }

        // Check generation (temporal safety)
        if id.generation != self.generation {
            return Err(ArenaErr::InvalidGeneration {
                expected: self.generation,
                found: id.generation,
            });
        }

        let id_end: usize = id.offset.to_usize() + id.size.to_usize();
        let offset_usize: usize = self.offset.to_usize();

        // Bounds check
        if id_end > offset_usize {
            return Err(ArenaErr::InvalidBounds);
        }

        // Size check
        if id.size.to_usize() != size_of::<T>() {
            return Err(ArenaErr::InvalidBounds);
        }

        Ok(())
    }

    /// Get reference to value (safe - checks generation and arena)
    #[inline]
    pub fn get<T>(&self, id: &ArenaId<T, I, M>) -> ArenaRes<&T> {
        self.validate_id(id)?;
        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset.to_usize()) as *const T;
            Ok(&*ptr)
        }
    }

    /// Get mutable reference to value (safe - checks generation and arena)
    #[inline]
    pub fn get_mut<T>(&mut self, id: &ArenaId<T, I, M>) -> ArenaRes<&mut T> {
        self.validate_id(id)?;
        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset.to_usize()) as *mut T;
            Ok(&mut *ptr)
        }
    }

    /// Get reference to value (unsafe - no generation check)

    /// Validate a Slice for safe access
    #[inline]
    fn validate_slice<T>(&self, slice: &Slice<T, I, M>) -> ArenaRes<()> {
        // Check arena ID first (cross-arena safety)
        if slice.arena_id != self.arena_id {
            return Err(ArenaErr::CrossArenaAccess {
                expected_id: self.arena_id,
                found_id: slice.arena_id,
            });
        }

        // Check generation (temporal safety)
        if slice.generation != self.generation {
            return Err(ArenaErr::InvalidGeneration {
                expected: self.generation,
                found: slice.generation,
            });
        }

        let slice_end: usize = slice.offset.to_usize() + slice.len.to_usize() * size_of::<T>();
        let offset_usize: usize = self.offset.to_usize();

        // Bounds check
        if slice_end > offset_usize {
            return Err(ArenaErr::InvalidBounds);
        }

        Ok(())
    }

    /// Get slice as slice (safe - checks generation and arena)
    #[inline]
    pub fn get_slice<T>(&self, slice: &Slice<T, I, M>) -> ArenaRes<&[T]> {
        self.validate_slice(slice)?;

        if slice.len.to_usize() == 0 {
            return Ok(&[]);
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            Ok(core::slice::from_raw_parts(ptr, slice.len.to_usize()))
        }
    }

    /// Get slice as mutable slice (safe - checks generation and arena)
    #[inline]
    pub fn get_slice_mut<T>(&mut self, slice: &Slice<T, I, M>) -> ArenaRes<&mut [T]> {
        self.validate_slice(slice)?;

        if slice.len.to_usize() == 0 {
            return Ok(&mut []);
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(slice.offset.to_usize()) as *mut T;
            Ok(core::slice::from_raw_parts_mut(ptr, slice.len.to_usize()))
        }
    }

    /// Clear arena (doesn't drop values!)
    pub fn clear(&mut self) {
        self.offset = I::try_from(0).unwrap_or_else(|_| panic!("I too small"));
        self.tail_offset = I::try_from(LEN).unwrap_or_else(|_| panic!("I too small for LEN"));
        self.generation = self.generation.wrapping_add(1);
        self.last_alloc = None;
    }

    /// Bytes used
    pub fn used(&self) -> usize {
        self.offset.to_usize()
    }

    /// Bytes remaining
    pub fn remaining(&self) -> usize {
        LEN - self.offset.to_usize()
    }

    /// Total Bytes
    pub fn capacity(&self) -> usize {
        LEN
    }

    /// Current generation
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Restore arena to a previous state (safe - invalidates handles)
    /// All handles created after this point become invalid
    pub fn restore_to(&mut self, offset: usize) {
        if offset <= LEN {
            self.offset = I::try_from(offset).unwrap_or_else(|_| panic!("Invalid restore offset"));
            self.generation = self.generation.wrapping_add(1);
            self.last_alloc = None;
        }
    }

    /// Get this arena's unique ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Pop the last allocation, invalidating its ID
    pub fn pop(&mut self) -> bool {
        if let Some(last) = self.last_alloc.take() {
            // Validate it's actually the last allocation
            if last.offset.to_usize() + last.size.to_usize() == self.offset.to_usize() {
                // Roll back
                self.offset = last.offset;
                self.generation = self.generation.wrapping_add(1);
                true
            } else {
                // Something's wrong - restore the tracking
                self.last_alloc = Some(last);
                false
            }
        } else {
            false
        }
    }

    // Iterators
    pub fn iter_slice<T>(&self, slice: &Slice<T, I, M>) -> ArenaRes<Iter<'_, T>> {
        self.validate_slice(slice)?;

        if slice.len.to_usize() == 0 {
            return Ok([].iter());
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            let slice_ref = core::slice::from_raw_parts(ptr, slice.len.to_usize());
            Ok(slice_ref.iter())
        }
    }

    pub fn iter_slice_range<T>(
        &self,
        slice: &Slice<T, I, M>,
        start: I,
        end: I,
    ) -> ArenaRes<Iter<'_, T>> {
        self.validate_slice(slice)?;

        let start_usize: usize = start.to_usize();
        let end_usize: usize = end.to_usize();
        let len_usize: usize = slice.len.to_usize();

        if start_usize > end_usize || end_usize > len_usize {
            return Err(ArenaErr::InvalidBounds);
        }

        if start_usize == end_usize {
            return Ok([].iter());
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            let slice_ref = core::slice::from_raw_parts(ptr, len_usize);
            Ok(slice_ref[start_usize..end_usize].iter())
        }
    }

    /// Internal method to allocate from tail (backwards from end)
    fn tail_alloc_bytes(&mut self, size: I) -> ArenaRes<I> {
        let size_usize = size.to_usize();
        let current_tail = self.tail_offset.to_usize();
        let front_used = self.offset.to_usize();

        if current_tail < size_usize || (current_tail - size_usize) < front_used {
            return Err(ArenaErr::OutOfSpace {
                requested: size_usize,
                available: current_tail - front_used,
            });
        }

        let new_tail_offset = current_tail - size_usize;
        self.tail_offset = I::try_from(new_tail_offset).map_err(|_| ArenaErr::IndexConversion)?;
        Ok(I::try_from(new_tail_offset).map_err(|_| ArenaErr::IndexConversion)?)
    }

    /// Internal method to copy slice data via tail allocation
    pub(crate) fn copy_slice_via_tail<T>(
        &mut self,
        source_slice: &Slice<T, I, M>,
        used_len: I,
    ) -> ArenaRes<Slice<T, I, M>>
    where
        T: Copy,
    {
        let size_bytes = I::try_from(used_len.to_usize() * core::mem::size_of::<T>())
            .map_err(|_| ArenaErr::IndexConversion)?;

        // Validate slice first
        self.validate_slice(source_slice)?;

        // Get source pointer directly to avoid borrow conflicts
        let source_start = source_slice.offset().to_usize();
        let source_ptr = unsafe { self.storage.as_ptr().add(source_start) as *const T };

        // Allocate tail space for temporary copy
        let tail_start = self.tail_alloc_bytes(size_bytes)?;
        let tail_ptr = unsafe { self.storage.as_mut_ptr().add(tail_start.to_usize()) as *mut T };

        // Copy to tail using raw pointers
        for i in 0..used_len.to_usize() {
            unsafe {
                let item = source_ptr.add(i).read();
                tail_ptr.add(i).write(item);
            }
        }

        // Allocate final destination normally
        let dest_slice =
            self.alloc_slice_from_fn(used_len.to_usize(), |i| unsafe { tail_ptr.add(i).read() })?;

        // Clear tail allocation
        self.tail_offset = I::try_from(LEN).map_err(|_| ArenaErr::IndexConversion)?;

        Ok(dest_slice)
    }
}

// Default implementation
impl<const LEN: usize, I, M> Default for Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_wraparound() {
        let mut arena: Arena<1024> = Arena::new();

        // Set generation near max
        arena.generation = u32::MAX - 1;

        let id1 = arena.alloc(42u32).unwrap();
        assert_eq!(id1.generation(), u32::MAX - 1);

        // This should wrap around
        arena.clear();
        assert_eq!(arena.generation(), u32::MAX);

        arena.clear();
        assert_eq!(arena.generation(), 0); // Wrapped around

        // Old ID should be invalid
        assert!(arena.get(&id1).is_err());
    }
}
