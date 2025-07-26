#![no_std]

//! Fixed-size arena allocator.

pub mod id;
pub mod pool;

#[cfg(test)]
mod tests;

use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;

pub use pool::Pool;
pub use id::{ArenaId, RawId};

/// Fixed-size arena. LEN = bytes, SizeType = handle size.
#[repr(C, align(16))]
#[derive(Debug)]
pub struct Arena<const LEN: usize, SizeType = usize> {
    /// Raw storage for all allocations
    storage: [MaybeUninit<u8>; LEN],
    /// Current allocation offset (bump pointer)
    offset: SizeType,
    /// Number of allocations made (for debugging/stats)
    count: SizeType,
}

impl<const LEN: usize, SizeType> Arena<LEN, SizeType>
where
    SizeType: Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = SizeType>,
{
    pub fn new() -> Self {
        // MaybeUninit doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small")),
            count: SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small")),
        }
    }

    /// Allocate and store a value
    pub fn alloc<T>(&mut self, value: T) -> Option<ArenaId<T, SizeType>>
    where
        T: 'static,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let offset_usize: usize = self.offset.into();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset = if misalignment != 0 {
            offset_usize + align - misalignment
        } else {
            offset_usize
        };

        // Check space
        if aligned_offset + size > LEN {
            return None;
        }

        self.offset = SizeType::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Store value
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            ptr::write(dst, value);
        }

        let id = ArenaId::new(self.offset, SizeType::try_from(size).ok()?);

        self.offset = self.offset + SizeType::try_from(size).ok()?;
        self.count = self.count + SizeType::try_from(1).ok()?;

        Some(id)
    }

    /// Allocate pool with initialization function
    pub fn alloc_pool_from_fn<T, F>(&mut self, count: usize, mut f: F) -> Option<Pool<T, SizeType>>
    where
        F: FnMut(usize) -> T,
    {
        if count == 0 {
            return Some(Pool::new(self.offset, SizeType::try_from(0).ok()?));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * count;
        let offset_usize: usize = self.offset.into();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset = if misalignment != 0 {
            offset_usize + align - misalignment
        } else {
            offset_usize
        };

        // Check space
        if aligned_offset + total_size > LEN {
            return None;
        }

        self.offset = SizeType::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Initialize elements
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            for i in 0..count {
                ptr::write(dst.add(i), f(i));
            }
        }

        let pool = Pool::new(self.offset, SizeType::try_from(count).ok()?);

        self.offset = self.offset + SizeType::try_from(total_size).ok()?;
        self.count = self.count + SizeType::try_from(1).ok()?;

        Some(pool)
    }

    /// Allocate pool with default values
    pub fn alloc_pool<T>(&mut self, count: usize) -> Option<Pool<T, SizeType>>
    where
        T: Default,
    {
        self.alloc_pool_from_fn(count, |_| T::default())
    }

    /// Get reference to value
    #[inline]
    pub fn get<T>(&self, id: &ArenaId<T, SizeType>) -> &T {
        let id_end: usize = id.offset.into() + id.size.into();
        let offset_usize: usize = self.offset.into();
        debug_assert!(id_end <= offset_usize, "Invalid ArenaId: out of bounds");
        debug_assert!(id.size.into() == size_of::<T>(), "Invalid ArenaId: size mismatch");

        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset.into()) as *const T;
            &*ptr
        }
    }

    /// Get mutable reference to value
    #[inline]
    pub fn get_mut<T>(&mut self, id: &ArenaId<T, SizeType>) -> &mut T {
        let id_end: usize = id.offset.into() + id.size.into();
        let offset_usize: usize = self.offset.into();
        debug_assert!(id_end <= offset_usize, "Invalid ArenaId: out of bounds");
        debug_assert!(id.size.into() == size_of::<T>(), "Invalid ArenaId: size mismatch");

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset.into()) as *mut T;
            &mut *ptr
        }
    }

    /// Get pool as slice
    #[inline]
    pub fn get_pool<T>(&self, pool: &Pool<T, SizeType>) -> &[T] {
        let pool_end: usize = pool.offset.into() + pool.len.into() * size_of::<T>();
        let offset_usize: usize = self.offset.into();
        debug_assert!(pool_end <= offset_usize, "Invalid Pool: out of bounds");

        if pool.len.into() == 0 {
            return &[];
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(pool.offset.into()) as *const T;
            core::slice::from_raw_parts(ptr, pool.len.into())
        }
    }

    /// Get pool as mutable slice
    #[inline]
    pub fn get_pool_mut<T>(&mut self, pool: &Pool<T, SizeType>) -> &mut [T] {
        let pool_end: usize = pool.offset.into() + pool.len.into() * size_of::<T>();
        let offset_usize: usize = self.offset.into();
        debug_assert!(pool_end <= offset_usize, "Invalid Pool: out of bounds");

        if pool.len.into() == 0 {
            return &mut [];
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(pool.offset.into()) as *mut T;
            core::slice::from_raw_parts_mut(ptr, pool.len.into())
        }
    }

    /// Clear arena (doesn't drop values!)
    pub fn clear(&mut self) {
        self.offset = SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small"));
        self.count = SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small"));
    }

    /// Bytes used
    pub fn used(&self) -> usize {
        self.offset.into()
    }

    /// Bytes remaining
    pub fn remaining(&self) -> usize {
        LEN - self.offset.into()
    }

    /// Number of allocations
    pub fn allocation_count(&self) -> usize {
        self.count.into()
    }

    /// Restore arena to a previous state (for checkpoint/restore)
    /// Warning: This doesn't drop values - caller must ensure safety!
    pub unsafe fn restore_to(&mut self, offset: usize) {
        self.offset = SizeType::try_from(offset).unwrap_or_else(|_| panic!("Invalid restore offset"));
        // Note: Not resetting count as it tracks total allocations made
    }
}



// Default implementation
impl<const LEN: usize, SizeType> Default for Arena<LEN, SizeType>
where
    SizeType: Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = SizeType>,
{
    fn default() -> Self {
        Self::new()
    }
}
