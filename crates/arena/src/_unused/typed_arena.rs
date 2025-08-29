//! Single-type arena with array-based allocation.
//! Faster than dynamic Arena for single types.
//! API matches Arena where possible: `alloc()`, `get()`, `clear()`, etc.
//! Key differences: `alloc_many()` vs `alloc_slice()`, `as_slice()` vs `get_slice()`.

use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::ArenaIndex;

// We'll use our own counter for typed arenas to avoid conflicts
static TYPED_ARENA_ID_COUNTER: AtomicU16 = AtomicU16::new(10000);

/// Handle to a value in a typed arena
#[derive(Debug, Clone, Copy, Hash)]
pub struct TypedId<T, Idx = u32, Marker = ()> {
    /// Array index (not byte offset)
    index: Idx,
    /// Generation when this ID was created
    generation: u16,
    /// Arena ID for cross-arena safety
    arena_id: u16,
    /// Zero-sized type marker
    _phantom: PhantomData<(T, Marker)>,
}

impl<T, Idx, Marker> TypedId<T, Idx, Marker> {
    /// Create a new TypedId (internal use)
    pub(crate) fn new(index: Idx, generation: u16, arena_id: u16) -> Self {
        Self { index, generation, arena_id, _phantom: PhantomData }
    }

    /// Get array index
    pub fn index(&self) -> usize
    where
        Idx: ArenaIndex,
    {
        self.index.into()
    }

    /// Get generation
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Get arena ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }
}

/// TypedIds are equal if they have the same index, generation, and arena_id
impl<T, Idx, Marker> PartialEq for TypedId<T, Idx, Marker>
where
    Idx: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.generation == other.generation
            && self.arena_id == other.arena_id
    }
}

impl<T, Idx, Marker> Eq for TypedId<T, Idx, Marker> where Idx: Eq {}

/// Fixed-capacity typed arena optimized for single type
#[derive(Debug)]
pub struct TypedArena<T, const CAPACITY: usize, Idx = u32, Marker = ()> {
    /// Storage for exactly CAPACITY elements of type T
    storage: [MaybeUninit<T>; CAPACITY],
    /// Number of allocated elements
    len: Idx,
    /// Current generation - incremented on clear/restore
    generation: u16,
    /// Unique arena ID for cross-arena safety
    arena_id: u16,
    /// Zero-sized type marker
    _marker: PhantomData<Marker>,
}

impl<T, const CAPACITY: usize, Idx, Marker> TypedArena<T, CAPACITY, Idx, Marker>
where
    Idx: ArenaIndex,
{
    /// Create a new typed arena
    pub fn new() -> Self {
        // Use the same atomic counter as the main arena for consistency
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            len: Idx::try_from(0).unwrap_or_else(|_| panic!("Idx too small")),
            generation: 0,
            arena_id: TYPED_ARENA_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            _marker: PhantomData,
        }
    }

    /// Allocate and store a value - much simpler than dynamic arena
    pub fn alloc(&mut self, value: T) -> Option<TypedId<T, Idx, Marker>> {
        let len_usize: usize = self.len.into();

        // Simple capacity check - no alignment calculation needed
        if len_usize >= CAPACITY {
            return None;
        }

        // Store value using simple array indexing
        self.storage[len_usize] = MaybeUninit::new(value);

        let id = TypedId::new(self.len, self.generation, self.arena_id);

        // Increment length
        self.len = self.len + Idx::one();

        Some(id)
    }

    /// Allocate value using a closure
    pub fn alloc_with<F>(&mut self, f: F) -> Option<TypedId<T, Idx, Marker>>
    where
        F: FnOnce() -> T,
    {
        let len_usize: usize = self.len.into();

        // Simple capacity check
        if len_usize >= CAPACITY {
            return None;
        }

        // Store value using closure
        self.storage[len_usize] = MaybeUninit::new(f());

        let id = TypedId::new(self.len, self.generation, self.arena_id);

        // Increment length
        self.len = self.len + Idx::one();

        Some(id)
    }

    /// Allocate multiple default values
    pub fn alloc_many(&mut self, count: usize) -> Option<TypedId<T, Idx, Marker>>
    where
        T: Default,
    {
        let len_usize: usize = self.len.into();

        if len_usize + count > CAPACITY {
            return None;
        }

        // Store the starting index
        let start_id = TypedId::new(self.len, self.generation, self.arena_id);

        // Initialize all elements
        for i in 0..count {
            self.storage[len_usize + i] = MaybeUninit::new(T::default());
        }

        // Update length
        self.len = self.len + Idx::try_from(count).ok()?;

        Some(start_id)
    }

    /// Allocate multiple values with closure
    pub fn alloc_many_with<F>(&mut self, count: usize, mut f: F) -> Option<TypedId<T, Idx, Marker>>
    where
        F: FnMut(usize) -> T,
    {
        let len_usize: usize = self.len.into();

        if len_usize + count > CAPACITY {
            return None;
        }

        // Store the starting index
        let start_id = TypedId::new(self.len, self.generation, self.arena_id);

        // Initialize all elements with closure
        for i in 0..count {
            self.storage[len_usize + i] = MaybeUninit::new(f(i));
        }

        // Update length
        self.len = self.len + Idx::try_from(count).ok()?;

        Some(start_id)
    }

    /// Validate TypedId for safe access
    #[inline]
    fn validate_id(&self, id: &TypedId<T, Idx, Marker>) -> bool {
        // Check arena ID first (cross-arena safety)
        if id.arena_id != self.arena_id {
            return false;
        }

        // Check generation (temporal safety)
        if id.generation != self.generation {
            return false;
        }

        // Simple bounds check
        let index: usize = id.index.into();
        let len: usize = self.len.into();
        index < len
    }

    /// Get reference to value
    #[inline]
    pub fn get(&self, id: &TypedId<T, Idx, Marker>) -> Option<&T> {
        if !self.validate_id(id) {
            return None;
        }

        // Simple array access - no pointer arithmetic
        unsafe { Some(self.storage[id.index.into()].assume_init_ref()) }
    }

    /// Get mutable reference to value
    #[inline]
    pub fn get_mut(&mut self, id: &TypedId<T, Idx, Marker>) -> Option<&mut T> {
        if !self.validate_id(id) {
            return None;
        }

        unsafe { Some(self.storage[id.index.into()].assume_init_mut()) }
    }

    /// Get reference (unsafe - no checks)
    #[inline]
    pub unsafe fn get_unchecked(&self, id: &TypedId<T, Idx, Marker>) -> &T {
        debug_assert_eq!(id.arena_id, self.arena_id, "Arena ID mismatch");
        debug_assert_eq!(id.generation, self.generation, "Generation mismatch");
        debug_assert!(id.index.into() < self.len.into(), "Index out of bounds");

        unsafe { self.storage[id.index.into()].assume_init_ref() }
    }

    /// Get mutable reference (unsafe - no checks)
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, id: &TypedId<T, Idx, Marker>) -> &mut T {
        debug_assert_eq!(id.arena_id, self.arena_id, "Arena ID mismatch");
        debug_assert_eq!(id.generation, self.generation, "Generation mismatch");
        debug_assert!(id.index.into() < self.len.into(), "Index out of bounds");

        unsafe { self.storage[id.index.into()].assume_init_mut() }
    }

    /// Get slice (unsafe - no checks)
    #[inline]
    pub unsafe fn as_slice_unchecked(&self) -> &[T] {
        let len: usize = self.len.into();
        if len == 0 {
            return &[];
        }

        unsafe {
            let ptr = self.storage.as_ptr() as *const T;
            core::slice::from_raw_parts(ptr, len)
        }
    }

    /// Get mutable slice (unsafe - no checks)
    #[inline]
    pub unsafe fn as_slice_unchecked_mut(&mut self) -> &mut [T] {
        let len: usize = self.len.into();
        if len == 0 {
            return &mut [];
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr() as *mut T;
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }

    /// Get slice of all allocated elements
    pub fn as_slice(&self) -> &[T] {
        let len: usize = self.len.into();
        if len == 0 {
            return &[];
        }

        unsafe {
            let ptr = self.storage.as_ptr() as *const T;
            core::slice::from_raw_parts(ptr, len)
        }
    }

    /// Get mutable slice of all allocated elements
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        let len: usize = self.len.into();
        if len == 0 {
            return &mut [];
        }

        unsafe {
            let ptr = self.storage.as_mut_ptr() as *mut T;
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }

    /// Clear arena (doesn't drop values)
    pub fn clear(&mut self) {
        self.len = Idx::try_from(0).unwrap_or_else(|_| panic!("Idx too small"));
        self.generation = self.generation.wrapping_add(1);
    }

    /// Clear arena and drop all values
    pub fn clear_and_drop(&mut self)
    where
        T: Drop,
    {
        let len: usize = self.len.into();

        // Drop all initialized values
        for i in 0..len {
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }

        self.clear();
    }

    /// Number of elements allocated
    pub fn len(&self) -> usize {
        self.len.into()
    }

    /// Number of elements allocated (alias for len)
    pub fn used(&self) -> usize {
        self.len.into()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len.into() == 0
    }

    /// Remaining capacity
    pub fn remaining(&self) -> usize {
        CAPACITY - self.len.into()
    }

    /// Total capacity
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Current generation
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Total size in bytes of all allocated elements
    pub fn used_bytes(&self) -> usize {
        self.len.into() * core::mem::size_of::<T>()
    }

    /// Remaining capacity in bytes
    pub fn remaining_bytes(&self) -> usize {
        self.remaining() * core::mem::size_of::<T>()
    }

    /// Total capacity in bytes
    pub fn capacity_bytes(&self) -> usize {
        CAPACITY * core::mem::size_of::<T>()
    }

    /// Restore to previous length
    pub fn restore_to(&mut self, len: usize) {
        if len <= CAPACITY && len <= self.len.into() {
            if let Ok(new_len) = Idx::try_from(len) {
                self.len = new_len;
                self.generation = self.generation.wrapping_add(1);
            }
        }
    }

    /// Check if handle is valid
    pub fn is_valid(&self, id: &TypedId<T, Idx, Marker>) -> bool {
        self.validate_id(id)
    }

    /// Get arena's unique ID
    pub fn arena_id(&self) -> u16 {
        self.arena_id
    }

    /// Iterator over elements with IDs
    pub fn iter_with_ids(&self) -> impl Iterator<Item = (TypedId<T, Idx, Marker>, &T)> {
        let len: usize = self.len.into();
        (0..len).filter_map(move |i| {
            if let Ok(index) = Idx::try_from(i) {
                let id = TypedId::new(index, self.generation, self.arena_id);
                let value = unsafe { self.storage[i].assume_init_ref() };
                Some((id, value))
            } else {
                None
            }
        })
    }
}

impl<T, const CAPACITY: usize, Idx, Marker> Default for TypedArena<T, CAPACITY, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        let id1 = arena.alloc(42).unwrap();
        let id2 = arena.alloc(84).unwrap();

        assert_eq!(*arena.get(&id1).unwrap(), 42);
        assert_eq!(*arena.get(&id2).unwrap(), 84);
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_capacity_limits() {
        let mut arena: TypedArena<u32, 2> = TypedArena::new();

        assert_eq!(arena.capacity(), 2);
        assert_eq!(arena.remaining(), 2);

        let _id1 = arena.alloc(1).unwrap();
        assert_eq!(arena.remaining(), 1);

        let _id2 = arena.alloc(2).unwrap();
        assert_eq!(arena.remaining(), 0);

        // Should fail - no capacity
        assert!(arena.alloc(3).is_none());
    }

    #[test]
    fn test_generational_safety() {
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        let id = arena.alloc(42).unwrap();
        assert_eq!(*arena.get(&id).unwrap(), 42);

        let gen_before = arena.generation();
        arena.clear();
        assert_eq!(arena.generation(), gen_before + 1);

        // Old ID should be invalid
        assert!(arena.get(&id).is_none());
        assert!(!arena.is_valid(&id));
    }

    #[test]
    fn test_slice_access() {
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        arena.alloc(10).unwrap();
        arena.alloc(20).unwrap();
        arena.alloc(30).unwrap();

        let slice = arena.as_slice();
        assert_eq!(slice, &[10, 20, 30]);

        arena.as_slice_mut()[1] = 200;
        assert_eq!(arena.as_slice(), &[10, 200, 30]);
    }

    #[test]
    fn test_batch_allocation() {
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        let _start_id = arena.alloc_many(5).unwrap();
        assert_eq!(arena.len(), 5);

        // All should be default values (0)
        for i in 0..5 {
            let id = TypedId::new(u16::try_from(i).unwrap(), arena.generation(), arena.arena_id());
            assert_eq!(*arena.get(&id).unwrap(), 0);
        }
    }

    #[test]
    fn test_type_markers() {
        struct MarkerA;
        struct MarkerB;

        let mut arena_a: TypedArena<u32, 100, u16, MarkerA> = TypedArena::new();
        let mut arena_b: TypedArena<u32, 100, u16, MarkerB> = TypedArena::new();

        let id_a = arena_a.alloc(42).unwrap();
        let id_b = arena_b.alloc(100).unwrap();

        assert_eq!(*arena_a.get(&id_a).unwrap(), 42);
        assert_eq!(*arena_b.get(&id_b).unwrap(), 100);

        // These should not compile due to type mismatch:
        // arena_a.get(&id_b); // Compile error!
        // arena_b.get(&id_a); // Compile error!
    }

    #[test]
    fn test_api_consistency() {
        // Test that TypedArena API mirrors Arena API as much as possible
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        // Basic allocation
        let id1 = arena.alloc(42).unwrap();
        let id2 = arena.alloc_with(|| 84).unwrap();

        // Access methods
        assert_eq!(*arena.get(&id1).unwrap(), 42);
        assert_eq!(*arena.get(&id2).unwrap(), 84);
        *arena.get_mut(&id1).unwrap() = 100;
        assert_eq!(*arena.get(&id1).unwrap(), 100);

        // Unsafe access
        unsafe {
            assert_eq!(*arena.get_unchecked(&id2), 84);
            *arena.get_unchecked_mut(&id2) = 200;
            assert_eq!(*arena.get_unchecked(&id2), 200);
        }

        // Stats methods (should match Arena API naming)
        assert_eq!(arena.len(), 2);
        assert_eq!(arena.used(), 2); // Alias for len()
        assert_eq!(arena.remaining(), 98);
        assert_eq!(arena.capacity(), 100);
        assert_eq!(arena.used_bytes(), 8); // 2 * 4 bytes
        assert_eq!(arena.remaining_bytes(), 392); // 98 * 4 bytes
        assert_eq!(arena.capacity_bytes(), 400); // 100 * 4 bytes

        // Generation and validity
        let gen1 = arena.generation();
        assert!(arena.is_valid(&id1));
        assert!(arena.is_valid(&id2));

        // Slice access
        let slice = arena.as_slice();
        assert_eq!(slice, &[100, 200]);
        arena.as_slice_mut()[0] = 500;
        assert_eq!(arena.as_slice()[0], 500);

        // Batch allocation
        let _batch_id = arena.alloc_many(3).unwrap();
        assert_eq!(arena.len(), 5);

        let _batch_id2 = arena.alloc_many_with(2, |i| (i + 10) as u32).unwrap();
        assert_eq!(arena.len(), 7);
        assert_eq!(arena.as_slice(), &[500, 200, 0, 0, 0, 10, 11]);

        // Clear and generation increment
        arena.clear();
        assert_eq!(arena.generation(), gen1 + 1);
        assert_eq!(arena.len(), 0);
        assert!(!arena.is_valid(&id1)); // Old IDs invalid
        assert!(arena.is_empty());

        // Arena ID consistency
        let arena_id = arena.arena_id();
        let new_id = arena.alloc(999).unwrap();
        assert_eq!(new_id.arena_id(), arena_id);
    }

    #[test]
    fn test_restore_functionality() {
        let mut arena: TypedArena<u32, 100> = TypedArena::new();

        let id1 = arena.alloc(1).unwrap();
        let id2 = arena.alloc(2).unwrap();
        let checkpoint = arena.len();
        let gen_before = arena.generation();

        let id3 = arena.alloc(3).unwrap();
        let id4 = arena.alloc(4).unwrap();

        assert_eq!(arena.len(), 4);
        assert_eq!(arena.as_slice(), &[1, 2, 3, 4]);

        // Restore to checkpoint
        arena.restore_to(checkpoint);

        // Generation should increment
        assert_eq!(arena.generation(), gen_before + 1);
        assert_eq!(arena.len(), 2);

        // All IDs become invalid due to generation change
        assert!(!arena.is_valid(&id1));
        assert!(!arena.is_valid(&id2));
        assert!(!arena.is_valid(&id3));
        assert!(!arena.is_valid(&id4));
    }
}
