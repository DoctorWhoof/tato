#![no_std]

//! A zero-allocation, fixed-size arena allocator.

pub mod id;
pub mod pool;

#[cfg(test)]
mod tests;

use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;

pub use pool::Pool;
pub use id::ArenaId;

/// A fixed-size, heterogeneous arena optimized for old-school constraints.
/// LEN is the storage size in bytes, SizeType controls handle size.
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
        // Safety: MaybeUninit<u8> doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small")),
            count: SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small")),
        }
    }

    /// Allocate space for a value of type T and store it
    pub fn alloc<T>(&mut self, value: T) -> Option<ArenaId<T, SizeType>>
    where
        T: 'static,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();
        let offset_usize: usize = self.offset.into();

        // Align the offset
        let misalignment = offset_usize % align;
        let aligned_offset = if misalignment != 0 {
            offset_usize + align - misalignment
        } else {
            offset_usize
        };

        // Check if we have enough space
        if aligned_offset + size > LEN {
            return None;
        }

        self.offset = SizeType::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Store the value
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            ptr::write(dst, value);
        }

        let id = ArenaId::new(self.offset, SizeType::try_from(size).ok()?);

        self.offset = self.offset + SizeType::try_from(size).ok()?;
        self.count = self.count + SizeType::try_from(1).ok()?;

        Some(id)
    }

    /// Allocate pool using closure to initialize each element
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

        // Align the offset
        let misalignment = offset_usize % align;
        let aligned_offset = if misalignment != 0 {
            offset_usize + align - misalignment
        } else {
            offset_usize
        };

        // Check if we have enough space
        if aligned_offset + total_size > LEN {
            return None;
        }

        self.offset = SizeType::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Initialize each element using the closure
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

    /// Allocate pool with Default::default() for each element
    pub fn alloc_pool<T>(&mut self, count: usize) -> Option<Pool<T, SizeType>>
    where
        T: Default,
    {
        self.alloc_pool_from_fn(count, |_| T::default())
    }

    /// Get a reference to the allocated value
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

    /// Get a mutable reference to the allocated value
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

    /// Get a slice reference to the allocated pool
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

    /// Get a mutable slice reference to the allocated pool
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

    /// Reset the arena to empty state (doesn't drop values - use carefully!)
    pub fn clear(&mut self) {
        self.offset = SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small"));
        self.count = SizeType::try_from(0).unwrap_or_else(|_| panic!("SizeType too small"));
    }

    /// Get the current number of bytes used
    pub fn used(&self) -> usize {
        self.offset.into()
    }

    /// Get the number of bytes remaining
    pub fn remaining(&self) -> usize {
        LEN - self.offset.into()
    }

    /// Get the number of allocations made
    pub fn allocation_count(&self) -> usize {
        self.count.into()
    }
}



// Convenience implementations
impl<const LEN: usize, SizeType> Default for Arena<LEN, SizeType>
where
    SizeType: Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = SizeType>,
{
    fn default() -> Self {
        Self::new()
    }
}
