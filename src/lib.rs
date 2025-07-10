#![no_std]

//! A no-allocation, fixed-size arena allocator.

pub mod id;
pub mod pool;

#[cfg(test)]
mod tests;

use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;

pub use pool::Pool;
pub use id::ArenaId;

/// A fixed-size, heterogeneous arena optimized for old-school constraints.
/// LEN is the storage size in bytes.
#[derive(Debug)]
pub struct Arena<const LEN: usize> {
    /// Raw storage for all allocations
    storage: [MaybeUninit<u8>; LEN],
    /// Current allocation offset (bump pointer)
    offset: usize,
    /// Number of allocations made (for debugging/stats)
    count: usize,
}

impl<const LEN: usize> Arena<LEN> {
    pub fn new() -> Self {
        // Safety: MaybeUninit<u8> doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: 0,
            count: 0,
        }
    }

    /// Allocate space for a value of type T and store it
    pub fn alloc<T>(&mut self, value: T) -> Option<ArenaId<T>>
    where
        T: 'static,
    {
        let size = size_of::<T>();
        let align = align_of::<T>();

        // Align the offset
        let misalignment = self.offset % align;
        if misalignment != 0 {
            self.offset += align - misalignment;
        }

        // Check if we have enough space
        if self.offset + size > LEN {
            return None;
        }

        // Store the value
        unsafe {
            let dst = self.storage.as_mut_ptr().add(self.offset) as *mut T;
            ptr::write(dst, value);
        }

        let id = ArenaId::new(self.offset, size);

        self.offset += size;
        self.count += 1;

        Some(id)
    }

    /// Allocate pool using closure to initialize each element
    pub fn alloc_pool_from_fn<T, F>(&mut self, count: usize, mut f: F) -> Option<Pool<T>>
    where
        F: FnMut(usize) -> T,
    {
        if count == 0 {
            return Some(Pool::new(self.offset, 0));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * count;

        // Align the offset
        let misalignment = self.offset % align;
        if misalignment != 0 {
            self.offset += align - misalignment;
        }

        // Check if we have enough space
        if self.offset + total_size > LEN {
            return None;
        }

        // Initialize each element using the closure
        unsafe {
            let dst = self.storage.as_mut_ptr().add(self.offset) as *mut T;
            for i in 0..count {
                ptr::write(dst.add(i), f(i));
            }
        }

        let pool = Pool::new(self.offset, count);

        self.offset += total_size;
        self.count += 1;

        Some(pool)
    }

    /// Allocate pool with Default::default() for each element
    pub fn alloc_pool<T>(&mut self, count: usize) -> Option<Pool<T>>
    where
        T: Default,
    {
        self.alloc_pool_from_fn(count, |_| T::default())
    }

    /// Get a reference to the allocated value
    #[inline]
    pub fn get<T>(&self, id: &ArenaId<T>) -> &T {
        debug_assert!(id.offset + id.size <= self.offset, "Invalid ArenaId: out of bounds");
        debug_assert!(id.size == size_of::<T>(), "Invalid ArenaId: size mismatch");

        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset) as *const T;
            &*ptr
        }
    }

    /// Get a mutable reference to the allocated value
    #[inline]
    pub fn get_mut<T>(&mut self, id: &ArenaId<T>) -> &mut T {
        debug_assert!(id.offset + id.size <= self.offset, "Invalid ArenaId: out of bounds");
        debug_assert!(id.size == size_of::<T>(), "Invalid ArenaId: size mismatch");

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset) as *mut T;
            &mut *ptr
        }
    }

    /// Get a slice reference to the allocated pool
    #[inline]
    pub fn get_pool<T>(&self, pool: &Pool<T>) -> &[T] {
        debug_assert!(pool.offset + pool.len * size_of::<T>() <= self.offset, "Invalid Pool: out of bounds");

        if pool.len == 0 {
            return &[];
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(pool.offset) as *const T;
            core::slice::from_raw_parts(ptr, pool.len)
        }
    }

    /// Get a mutable slice reference to the allocated pool
    #[inline]
    pub fn get_pool_mut<T>(&mut self, pool: &Pool<T>) -> &mut [T] {
        debug_assert!(pool.offset + pool.len * size_of::<T>() <= self.offset, "Invalid Pool: out of bounds");

        if pool.len == 0 {
            return &mut [];
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(pool.offset) as *mut T;
            core::slice::from_raw_parts_mut(ptr, pool.len)
        }
    }

    /// Reset the arena to empty state (doesn't drop values - use carefully!)
    pub fn clear(&mut self) {
        self.offset = 0;
        self.count = 0;
    }

    /// Get the current number of bytes used
    pub fn used(&self) -> usize {
        self.offset
    }

    /// Get the number of bytes remaining
    pub fn remaining(&self) -> usize {
        LEN - self.offset
    }

    /// Get the number of allocations made
    pub fn allocation_count(&self) -> usize {
        self.count
    }
}



// Convenience implementations
impl<const LEN: usize> Default for Arena<LEN> {
    fn default() -> Self {
        Self::new()
    }
}
