use core::marker::PhantomData;
use core::mem::{MaybeUninit, align_of, size_of};
use core::ptr;
use core::slice::Iter;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::{ArenaId, ArenaIndex, Slice};

// Global counter for unique arena IDs (no-std compatible)
static ARENA_ID_COUNTER: AtomicU16 = AtomicU16::new(1);

/// Fixed-size arena with generational safety.
/// LEN = bytes, Idx = handle size, Marker = type safety marker.
#[repr(C, align(16))]
#[derive(Debug)]
pub struct Arena<const LEN: usize, Idx = u16, Marker = ()> {
    /// Raw storage for all allocations
    storage: [MaybeUninit<u8>; LEN],
    /// Current allocation offset (bump pointer)
    offset: Idx,
    /// Current generation - incremented on restore_to()
    generation: u32,
    /// Unique arena ID for cross-arena safety
    arena_id: u16,
    /// Zero-sized type marker for compile-time arena safety
    _marker: PhantomData<Marker>,
}

impl<const LEN: usize, Idx, Marker> Arena<LEN, Idx, Marker>
where
    Idx: ArenaIndex,
{
    /// Create a new arena with automatic cross-arena safety.
    /// Each arena gets a unique ID from an atomic counter, ensuring perfect
    /// collision resistance and automatic cross-arena safety without requiring
    /// explicit marker types.
    pub fn new() -> Self {
        // Use atomic counter for guaranteed uniqueness
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: Idx::try_from(0).unwrap_or_else(|_| panic!("Idx too small")),
            generation: 0,
            arena_id: ARENA_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            _marker: PhantomData,
        }
    }

    /// Allocate and store a value
    pub fn alloc<T>(&mut self, value: T) -> Option<ArenaId<T, Idx, Marker>>
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
            return None;
        }

        self.offset = Idx::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Store value
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            ptr::write(dst, value);
        }

        let id =
            ArenaId::new(self.offset, Idx::try_from(size).ok()?, self.generation, self.arena_id);

        self.offset = self.offset + Idx::try_from(size).ok()?;

        Some(id)
    }

    /// Allocate slice with initialization function
    pub fn alloc_slice_from_fn<T, F>(
        &mut self,
        count: Idx,
        mut f: F,
    ) -> Option<Slice<T, Idx, Marker>>
    where
        F: FnMut(usize) -> T,
    {
        if count == Idx::zero() {
            return Some(Slice::new(
                self.offset,
                Idx::try_from(0).ok()?,
                self.generation,
                self.arena_id,
            ));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * count.to_usize();
        let offset_usize: usize = self.offset.to_usize();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset =
            if misalignment != 0 { offset_usize + align - misalignment } else { offset_usize };

        // Check space
        if aligned_offset + total_size > LEN {
            return None;
        }

        self.offset = Idx::try_from(aligned_offset).map_err(|_| ()).ok()?;

        // Initialize elements
        unsafe {
            let dst = self.storage.as_mut_ptr().add(aligned_offset) as *mut T;
            let count: usize = count.to_usize();
            for i in 0..count {
                ptr::write(dst.add(i), f(i));
            }
        }

        let slice = Slice::new(self.offset, count, self.generation, self.arena_id);

        self.offset = self.offset + Idx::try_from(total_size).ok()?;

        Some(slice)
    }

    /// Allocate slice with default values
    pub fn alloc_slice<T>(&mut self, count: Idx) -> Option<Slice<T, Idx, Marker>>
    where
        T: Default,
    {
        self.alloc_slice_from_fn(count, |_| T::default())
    }

    /// Allocate uninitialized slice
    pub fn alloc_slice_uninit<T>(&mut self, count: Idx) -> Option<Slice<T, Idx, Marker>> {
        if count == Idx::zero() {
            return Some(Slice::new(
                self.offset,
                Idx::try_from(0).ok()?,
                self.generation,
                self.arena_id,
            ));
        }

        let size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = size * count.to_usize();
        let offset_usize: usize = self.offset.to_usize();

        // Align offset
        let misalignment = offset_usize % align;
        let aligned_offset =
            if misalignment != 0 { offset_usize + align - misalignment } else { offset_usize };

        // Check space
        if aligned_offset + total_size > LEN {
            return None;
        }

        self.offset = Idx::try_from(aligned_offset).map_err(|_| ()).ok()?;

        let slice = Slice::new(self.offset, count, self.generation, self.arena_id);

        self.offset = self.offset + Idx::try_from(total_size).ok()?;

        Some(slice)
    }

    /// Allocate all of remaining space with a single slice
    pub fn fill_with_slice<T>(&mut self) -> Option<Slice<T, Idx, Marker>>
    where
        T: Default,
    {
        let count = Idx::from_usize_checked(self.remaining() / size_of::<T>()).unwrap();
        self.alloc_slice_from_fn(count, |_| T::default())
    }

    /// Validate an ArenaId for safe access
    #[inline]
    fn validate_id<T>(&self, id: &ArenaId<T, Idx, Marker>) -> bool {
        // Check arena ID first (cross-arena safety)
        if id.arena_id != self.arena_id {
            return false;
        }

        // Check generation (temporal safety)
        if id.generation != self.generation {
            return false;
        }

        let id_end: usize = id.offset.to_usize() + id.size.to_usize();
        let offset_usize: usize = self.offset.to_usize();

        // Bounds check
        if id_end > offset_usize {
            return false;
        }

        // Size check
        id.size.to_usize() == size_of::<T>()
    }

    /// Get reference to value (safe - checks generation and arena)
    #[inline]
    pub fn get<T>(&self, id: &ArenaId<T, Idx, Marker>) -> Option<&T> {
        if !self.validate_id(id) {
            return None;
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset.to_usize()) as *const T;
            Some(&*ptr)
        }
    }

    /// Get mutable reference to value (safe - checks generation and arena)
    #[inline]
    pub fn get_mut<T>(&mut self, id: &ArenaId<T, Idx, Marker>) -> Option<&mut T> {
        if !self.validate_id(id) {
            return None;
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset.to_usize()) as *mut T;
            Some(&mut *ptr)
        }
    }

    /// Get reference to value (unsafe - no generation check)
    /// Only use this if you're certain the handle is valid
    #[inline]
    pub unsafe fn get_unchecked<T>(&self, id: &ArenaId<T, Idx, Marker>) -> &T {
        debug_assert_eq!(id.arena_id, self.arena_id, "Arena ID mismatch in get_unchecked");
        debug_assert_eq!(id.generation, self.generation, "Generation mismatch in get_unchecked");
        unsafe {
            let ptr = self.storage.as_ptr().add(id.offset.to_usize()) as *const T;
            &*ptr
        }
    }

    /// Get mutable reference to value (unsafe - no generation check)
    /// Only use this if you're certain the handle is valid
    #[inline]
    pub unsafe fn get_unchecked_mut<T>(&mut self, id: &ArenaId<T, Idx, Marker>) -> &mut T {
        debug_assert_eq!(id.arena_id, self.arena_id, "Arena ID mismatch in get_unchecked_mut");
        debug_assert_eq!(
            id.generation, self.generation,
            "Generation mismatch in get_unchecked_mut"
        );
        unsafe {
            let ptr = self.storage.as_mut_ptr().add(id.offset.to_usize()) as *mut T;
            &mut *ptr
        }
    }

    /// Validate a Slice for safe access
    #[inline]
    fn validate_slice<T>(&self, slice: &Slice<T, Idx, Marker>) -> bool {
        // Check arena ID first (cross-arena safety)
        if slice.arena_id != self.arena_id {
            debug_assert!(
                false, // TODO: Should become an error result
                "Arena Error: Invalid arena id in handle"
            );
            return false;
        }

        // Check generation (temporal safety)
        if slice.generation != self.generation {
            debug_assert!(
                false, // TODO: Should become an error result
                "Arena Error: Invalid arena generation in handle"
            );
            return false;
        }

        let slice_end: usize = slice.offset.to_usize() + slice.len.to_usize() * size_of::<T>();
        let offset_usize: usize = self.offset.to_usize();

        // Bounds check
        let bounds_check = slice_end <= offset_usize;
        debug_assert!(
            bounds_check, // TODO: Should become an error result
            "Arena Error: Invalid bounds check, slice end must less or equal than slice start"
        );
        bounds_check
    }

    /// Get slice as slice (safe - checks generation and arena)
    #[inline]
    pub fn get_slice<T>(&self, slice: &Slice<T, Idx, Marker>) -> Option<&[T]> {
        if !self.validate_slice(slice) {
            return None;
        }

        if slice.len.to_usize() == 0 {
            return Some(&[]);
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            Some(core::slice::from_raw_parts(ptr, slice.len.to_usize()))
        }
    }

    /// Get slice as mutable slice (safe - checks generation and arena)
    #[inline]
    pub fn get_slice_mut<T>(&mut self, slice: &Slice<T, Idx, Marker>) -> Option<&mut [T]> {
        if !self.validate_slice(slice) {
            return None;
        }

        if slice.len.to_usize() == 0 {
            return Some(&mut []);
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(slice.offset.to_usize()) as *mut T;
            Some(core::slice::from_raw_parts_mut(ptr, slice.len.to_usize()))
        }
    }

    /// Get slice as slice (unsafe - no generation check)
    /// Only use this if you're certain the handle is valid
    #[inline]
    pub unsafe fn get_slice_unchecked<T>(&self, slice: &Slice<T, Idx, Marker>) -> &[T] {
        debug_assert_eq!(slice.arena_id, self.arena_id, "Arena ID mismatch in get_slice_unchecked");
        debug_assert_eq!(
            slice.generation, self.generation,
            "Generation mismatch in get_slice_unchecked"
        );

        if slice.len.to_usize() == 0 {
            return &[];
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            core::slice::from_raw_parts(ptr, slice.len.to_usize())
        }
    }

    /// Get slice as mutable slice (unsafe - no generation check)
    /// Only use this if you're certain the handle is valid
    #[inline]
    pub unsafe fn get_slice_unchecked_mut<T>(&mut self, slice: &Slice<T, Idx, Marker>) -> &mut [T] {
        debug_assert_eq!(
            slice.arena_id, self.arena_id,
            "Arena ID mismatch in get_slice_unchecked_mut"
        );
        debug_assert_eq!(
            slice.generation, self.generation,
            "Generation mismatch in get_slice_unchecked_mut"
        );

        if slice.len.to_usize() == 0 {
            return &mut [];
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr().add(slice.offset.to_usize()) as *mut T;
            core::slice::from_raw_parts_mut(ptr, slice.len.to_usize())
        }
    }

    /// Clear arena (doesn't drop values!)
    pub fn clear(&mut self) {
        self.offset = Idx::try_from(0).unwrap_or_else(|_| panic!("Idx too small"));
        self.generation = self.generation.wrapping_add(1);
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
            self.offset =
                Idx::try_from(offset).unwrap_or_else(|_| panic!("Invalid restore offset"));
            self.generation = self.generation.wrapping_add(1);
        }
    }

    /// Check if a handle is valid for this arena
    pub fn is_valid<T>(&self, id: &ArenaId<T, Idx, Marker>) -> bool {
        self.validate_id(id)
    }

    /// Check if a slice handle is valid for this arena
    pub fn is_slice_valid<T>(&self, slice: &Slice<T, Idx, Marker>) -> bool {
        self.validate_slice(slice)
    }

    /// Get this arena's unique ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    // Iterators
    pub fn iter_slice<T>(&self, slice: &Slice<T, Idx, Marker>) -> Option<Iter<'_, T>> {
        if !self.validate_slice(slice) {
            return None;
        }

        if slice.len.to_usize() == 0 {
            return Some([].iter());
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            let slice_ref = core::slice::from_raw_parts(ptr, slice.len.to_usize());
            Some(slice_ref.iter())
        }
    }

    pub fn iter_slice_range<T>(
        &self,
        slice: &Slice<T, Idx, Marker>,
        start: Idx,
        end: Idx,
    ) -> Option<Iter<'_, T>> {
        if !self.validate_slice(slice) {
            return None;
        }

        let start_usize: usize = start.to_usize();
        let end_usize: usize = end.to_usize();
        let len_usize: usize = slice.len.to_usize();

        if start_usize > end_usize || end_usize > len_usize {
            return None;
        }

        if start_usize == end_usize {
            return Some([].iter());
        }

        unsafe {
            let ptr = self.storage.as_ptr().add(slice.offset.to_usize()) as *const T;
            let slice_ref = core::slice::from_raw_parts(ptr, len_usize);
            Some(slice_ref[start_usize..end_usize].iter())
        }
    }
}

// Default implementation
impl<const LEN: usize, Idx, Marker> Default for Arena<LEN, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self::new()
    }
}

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
    assert!(arena.get(&id1).is_none());
}
