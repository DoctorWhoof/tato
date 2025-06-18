use core::any::TypeId;
use core::marker::PhantomData;
use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;

#[derive(Debug)]
struct TypeInfo {
    type_id: TypeId,
    size: usize,
    align: usize,
}

/// A simple allocation entry just for type tracking
#[derive(Debug)]
struct Allocation {
    type_info: TypeInfo,
}

/// A fixed-size, heterogeneous arena with length LEN measured in bytes.
#[derive(Debug)]
pub struct Arena<const LEN: usize, const ALLOC_COUNT: usize> {
    /// Storage for actual data values
    storage: [MaybeUninit<u8>; LEN],
    /// Fixed array of allocation types
    allocations: [Allocation; ALLOC_COUNT],
    /// Current number of allocations
    count: usize,
    /// Current offset in the storage (next allocation position)
    next_offset: usize,
}

impl<const LEN: usize, const ALLOC_COUNT: usize> Arena<LEN, ALLOC_COUNT> {
    pub fn new() -> Self {
        // Initialize empty allocations array with dummy values
        let allocations: [Allocation; ALLOC_COUNT] = core::array::from_fn(|_| Allocation {
            type_info: TypeInfo { type_id: TypeId::of::<()>(), size: 0, align: 1 },
        });

        // Safety: MaybeUninit<u8> doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };

        Self { storage, allocations, count: 0, next_offset: 0 }
    }

    pub fn alloc<T>(&mut self, value: T) -> Option<ArenaId<T>>
    where
        T: 'static,
    {
        // Check if we've reached the allocation limit
        if self.count >= ALLOC_COUNT {
            return None;
        }

        // Get type information
        let type_info = TypeInfo {
            type_id: TypeId::of::<T>(),
            size: size_of::<T>(),
            align: align_of::<T>(),
        };

        // Adjust next_offset for alignment
        let misalignment = self.next_offset % type_info.align;
        if misalignment != 0 {
            self.next_offset += type_info.align - misalignment;
        }

        // Check if there's enough space left
        if self.next_offset + type_info.size > LEN {
            return None;
        }

        // Write the value to the storage
        unsafe {
            let dst = self.storage.as_mut_ptr().add(self.next_offset) as *mut T;
            ptr::write(dst, value);
        }

        // Record the allocation and advance
        let alloc_idx = self.count;
        let size = type_info.size;
        self.allocations[alloc_idx] = Allocation { type_info };
        let offset = self.next_offset;

        // Advance the allocation pointer
        self.next_offset += size;
        self.count += 1;

        // Return an ID that can be used to access this value
        Some(ArenaId { index: alloc_idx, offset, _marker: PhantomData })
    }

    pub fn get<T: 'static>(&self, id: &ArenaId<T>) -> Option<&T> {
        if id.index >= self.count {
            return None;
        }

        let alloc = &self.allocations[id.index];
        if alloc.type_info.type_id == TypeId::of::<T>() {
            unsafe {
                let ptr = self.storage.as_ptr().add(id.offset) as *const T;
                Some(&*ptr)
            }
        } else {
            None // Type mismatch!
        }
    }

    pub fn get_mut<T: 'static>(&mut self, id: &ArenaId<T>) -> Option<&mut T> {
        if id.index >= self.count {
            return None;
        }

        let alloc = &self.allocations[id.index];
        if alloc.type_info.type_id == TypeId::of::<T>() {
            unsafe {
                let ptr = self.storage.as_mut_ptr().add(id.offset) as *mut T;
                Some(&mut *ptr)
            }
        } else {
            None // Type mismatch!
        }
    }

    /// Reset the arena to empty state
    pub fn clear(&mut self) {
        self.count = 0;
        self.next_offset = 0;
        // No need to clear the storage array - we'll just overwrite it on next use
    }
}

// A handle to an allocated value
#[derive(Debug, Clone)]
pub struct ArenaId<T> {
    index: usize,
    offset: usize, // Store the offset for direct access
    _marker: PhantomData<T>,
}
