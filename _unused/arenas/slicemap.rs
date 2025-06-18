use core::mem::{MaybeUninit, size_of, align_of};
use core::marker::PhantomData;
use core::ptr;

/// ID for a slice in the SliceMap
#[derive(Debug, Clone, Copy)]
pub struct SliceId<T> {
    /// Index in the allocations array
    index: usize,
    /// Length of the slice
    length: usize,
    /// Phantom data for the type
    _marker: PhantomData<T>,
}

/// Entry in the allocations array
struct SliceEntry {
    /// Start offset in the storage
    start: usize,
    /// Length in bytes
    byte_len: usize,
    /// Whether this entry is in use
    is_used: bool,
}

/// A map that stores slices of varying lengths
pub struct SliceMap<T, const STORAGE: usize, const ENTRIES: usize> {
    /// Raw storage for all elements
    storage: [MaybeUninit<u8>; STORAGE],
    /// Metadata about allocations
    allocations: [SliceEntry; ENTRIES],
    /// Current number of allocations
    count: usize,
    /// Next offset in storage
    next_offset: usize,
    /// Phantom data for the type
    _marker: PhantomData<T>,
}

impl<T: 'static, const STORAGE: usize, const ENTRIES: usize> SliceMap<T, STORAGE, ENTRIES> {
    /// Create a new empty SliceMap
    pub fn new() -> Self {
        // Initialize allocations array with empty entries
        let allocations = core::array::from_fn(|_| SliceEntry {
            start: 0,
            byte_len: 0,
            is_used: false,
        });

        // Safety: MaybeUninit<u8> doesn't need initialization
        let storage = unsafe { MaybeUninit::uninit().assume_init() };

        Self {
            storage,
            allocations,
            count: 0,
            next_offset: 0,
            _marker: PhantomData,
        }
    }

    /// Store a slice in the map
    pub fn store(&mut self, slice: &[T]) -> Option<SliceId<T>> {
        // Calculate size and alignment
        let align = align_of::<T>();
        let elem_size = size_of::<T>();
        let total_size = elem_size * slice.len();

        // Align offset
        let misalignment = self.next_offset % align;
        if misalignment != 0 {
            self.next_offset += align - misalignment;
        }

        // Check if we have enough space
        if self.next_offset + total_size > STORAGE || self.count >= ENTRIES {
            return None;
        }

        // Copy the slice to storage
        let offset = self.next_offset;
        for (i, item) in slice.iter().enumerate() {
            unsafe {
                let dst = self.storage.as_mut_ptr().add(offset + i * elem_size) as *mut T;
                ptr::write(dst, core::ptr::read(item));
            }
        }

        // Record allocation
        let index = self.count;
        self.allocations[index] = SliceEntry {
            start: offset,
            byte_len: total_size,
            is_used: true,
        };

        // Advance
        self.next_offset += total_size;
        self.count += 1;

        // Return ID
        Some(SliceId {
            index,
            length: slice.len(),
            _marker: PhantomData,
        })
    }

    /// Get a slice by ID
    pub fn get(&self, id: &SliceId<T>) -> Option<&[T]> {
        if id.index >= self.count || !self.allocations[id.index].is_used {
            return None;
        }

        let entry = &self.allocations[id.index];
        let ptr = unsafe {
            self.storage.as_ptr().add(entry.start) as *const T
        };

        unsafe {
            Some(core::slice::from_raw_parts(ptr, id.length))
        }
    }
}
