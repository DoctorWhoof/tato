use core::marker::PhantomData;
use core::mem::{MaybeUninit, size_of, align_of};
use core::ptr;

/// A fixed-size arena for a single type T with length LEN measured in bytes.
pub struct FixedArena<T, const LEN: usize> {
    /// Storage for elements, represented as bytes
    storage: [MaybeUninit<u8>; LEN],
    /// Current number of allocations
    count: usize,
    /// Next offset where we'll place an item
    next_offset: usize,
    /// Phantom data to track the type
    _marker: PhantomData<T>,
}

/// A handle to an allocated value in the arena
pub struct ArenaId {
    /// Index of the allocation (for bounds checking)
    index: usize,
    /// Byte offset in the storage array
    offset: usize,
}

impl<T: 'static, const LEN: usize> FixedArena<T, LEN> {
    pub fn new() -> Self {
        // Safety: MaybeUninit<u8> doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };

        Self {
            storage,
            count: 0,
            next_offset: 0,
            _marker: PhantomData,
        }
    }

    /// Return the maximum number of T that can fit in this arena
    pub fn capacity() -> usize {
        // Account for alignment padding in worst case
        LEN / (size_of::<T>() + align_of::<T>() - 1)
    }

    pub fn alloc(&mut self, value: T) -> Option<ArenaId> {
        // Calculate alignment requirements
        let align = align_of::<T>();
        let size = size_of::<T>();

        // Adjust offset for alignment
        let misalignment = self.next_offset % align;
        if misalignment != 0 {
            self.next_offset += align - misalignment;
        }

        // Check if there's enough space left
        if self.next_offset + size > LEN {
            return None;
        }

        // Write the value to the storage
        unsafe {
            let dst = self.storage.as_mut_ptr().add(self.next_offset) as *mut T;
            ptr::write(dst, value);
        }

        // Record allocation
        let index = self.count;
        let offset = self.next_offset;

        // Advance the allocation pointer and count
        self.next_offset += size;
        self.count += 1;

        // Return ID
        Some(ArenaId { index, offset })
    }

    pub fn get(&self, id: &ArenaId) -> Option<&T> {
        if id.index >= self.count {
            return None;
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset) as *const T;
            Some(&*ptr)
        }
    }

    pub fn get_mut(&mut self, id: &ArenaId) -> Option<&mut T> {
        if id.index >= self.count {
            return None;
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset) as *mut T;
            Some(&mut *ptr)
        }
    }

    /// Reset the arena to empty state
    pub fn clear(&mut self) {
        self.count = 0;
        self.next_offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn test_basic_allocation_and_retrieval() {
        let mut arena = FixedArena::<i32, 1024>::new();

        // Allocate values
        let id1 = arena.alloc(42).unwrap();
        let id2 = arena.alloc(100).unwrap();
        let id3 = arena.alloc(-5).unwrap();

        // Verify retrieval
        assert_eq!(*arena.get(&id1).unwrap(), 42);
        assert_eq!(*arena.get(&id2).unwrap(), 100);
        assert_eq!(*arena.get(&id3).unwrap(), -5);
    }

    #[test]
    fn test_mutability() {
        // Instead of String, use an array with a counter
        #[derive(PartialEq)]
        struct Counter {
            data: [u8; 8],
            len: Cell<usize>,
        }

        impl Counter {
            fn new() -> Self {
                Self {
                    data: [0; 8],
                    len: Cell::new(0),
                }
            }

            fn push(&self, value: u8) {
                let len = self.len.get();
                if len < 8 {
                    // Fixed: Use proper pointer arithmetic
                    unsafe {
                        let ptr = self.data.as_ptr() as *mut u8;
                        *ptr.add(len) = value;
                    }
                    self.len.set(len + 1);
                }
            }

            fn len(&self) -> usize {
                self.len.get()
            }
        }

        let mut arena = FixedArena::<Counter, 1024>::new();

        let id = arena.alloc(Counter::new()).unwrap();

        // Modify the value
        if let Some(counter) = arena.get(&id) {
            counter.push(42);
            counter.push(10);
        }

        // Verify it was modified
        let counter = arena.get(&id).unwrap();
        assert_eq!(counter.len(), 2);
        assert_eq!(counter.data[0], 42);
        assert_eq!(counter.data[1], 10);
    }

    #[test]
    fn test_capacity_limits() {
        // Create a tiny arena that can only hold two i32s
        // i32 is 4 bytes with 4-byte alignment, so we need at least 8 bytes
        let mut arena = FixedArena::<i32, 8>::new();

        // Should fit two i32s
        let _id1 = arena.alloc(1).unwrap();
        let _id2 = arena.alloc(2).unwrap();

        // Third allocation should fail
        assert!(arena.alloc(3).is_none());
    }

    #[test]
    fn test_clear() {
        let mut arena = FixedArena::<i32, 16>::new();

        // Fill the arena
        let _id1 = arena.alloc(1).unwrap();
        let _id2 = arena.alloc(2).unwrap();

        // Clear the arena
        arena.clear();

        // Should be able to allocate again
        let id_new = arena.alloc(10).unwrap();
        assert_eq!(*arena.get(&id_new).unwrap(), 10);

        // We should be able to allocate the same number as before
        let _id_new2 = arena.alloc(20).unwrap();
    }

    #[test]
    fn test_alignment_requirements() {
        // Create a type with 8-byte alignment
        #[repr(align(8))]
        struct Aligned(u32);

        // An arena with 15 bytes - enough for an Aligned plus padding
        let mut arena = FixedArena::<Aligned, 15>::new();

        // First allocation should succeed (8 bytes for alignment + 4 bytes for data)
        let id = arena.alloc(Aligned(42)).unwrap();
        assert_eq!(arena.get(&id).unwrap().0, 42);

        // Second allocation should fail (not enough space left)
        assert!(arena.alloc(Aligned(100)).is_none());
    }

    #[test]
    fn test_id_after_clear() {
        let mut arena = FixedArena::<i32, 64>::new();

        // Allocate, clear, then allocate again
        let id1 = arena.alloc(42).unwrap();
        arena.clear();

        // This ID refers to data that no longer exists after clear
        assert!(arena.get(&id1).is_none());

        // New allocations should work
        let id2 = arena.alloc(100).unwrap();
        assert_eq!(*arena.get(&id2).unwrap(), 100);
    }

    #[test]
    fn test_capacity_method() {
        // Calculate a minimum expected capacity for i32
        let size = core::mem::size_of::<i32>();
        let align = core::mem::align_of::<i32>();
        let min_expected = 1024 / (size + align);

        let capacity = FixedArena::<i32, 1024>::capacity();
        assert!(capacity > 0, "Capacity should be positive");
        assert!(capacity >= min_expected,
                "Capacity should be at least {} for i32 in 1024 bytes, got {}",
                min_expected, capacity);

        // Test with a larger aligned type - just ensure it returns something reasonable
        #[repr(align(16))]
        struct BigAligned([u8; 32]);

        let capacity = FixedArena::<BigAligned, 1024>::capacity();
        assert!(capacity > 0, "Capacity should be positive for aligned type");
    }

    #[test]
    fn test_zero_sized_type() {
        // Unit struct has zero size
        struct Unit;

        let mut arena = FixedArena::<Unit, 16>::new();

        // We should be able to allocate many of these
        for _ in 0..100 {
            let id = arena.alloc(Unit).unwrap();
            assert!(arena.get(&id).is_some());
        }
    }
}
