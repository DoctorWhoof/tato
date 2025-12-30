use super::*;

#[test]
fn test_arena_ref_basic_creation() {
    let mut arena: Arena<1024> = Arena::new();
    let arena_ref = arena.as_ref();
    
    // Should have same capacity
    assert_eq!(arena_ref.capacity(), 1024);
    assert_eq!(arena_ref.used(), 0);
    assert_eq!(arena_ref.remaining(), 1024);
}

#[test]
fn test_arena_ref_allocations() {
    let mut arena: Arena<1024> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Test basic allocation
    let id1 = arena_ref.alloc(42u32).unwrap();
    let id2 = arena_ref.alloc(3.14f64).unwrap();
    
    assert_eq!(*arena_ref.get(id1).unwrap(), 42u32);
    assert_eq!(*arena_ref.get(id2).unwrap(), 3.14f64);
    
    // Test mutable access
    *arena_ref.get_mut(id1).unwrap() = 100;
    assert_eq!(*arena_ref.get(id1).unwrap(), 100);
}

#[test]
fn test_arena_ref_slice_operations() {
    let mut arena: Arena<1024> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Test slice allocation
    let slice1 = arena_ref.alloc_slice(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(slice1.len(), 5);
    
    // Test slice from function
    let slice2 = arena_ref.alloc_slice_from_fn(3, |i| i * 10).unwrap();
    
    // Test slice access
    assert_eq!(arena_ref.get_slice(slice1).unwrap(), &[1, 2, 3, 4, 5]);
    assert_eq!(arena_ref.get_slice(slice2).unwrap(), &[0, 10, 20]);
    
    // Test mutable slice access
    {
        let slice_mut = arena_ref.get_slice_mut(slice1).unwrap();
        slice_mut[2] = 99;
    }
    assert_eq!(arena_ref.get_slice(slice1).unwrap(), &[1, 2, 99, 4, 5]);
}

#[test]
fn test_arena_ref_iteration() {
    let mut arena: Arena<1024> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    let slice = arena_ref.alloc_slice_from_fn(5, |i| i as u32 * 2).unwrap();
    
    // Test full iteration
    let mut iter = arena_ref.iter_slice(slice).unwrap();
    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&4));
    assert_eq!(iter.next(), Some(&6));
    assert_eq!(iter.next(), Some(&8));
    assert_eq!(iter.next(), None);
    
    // Test range iteration
    let mut range_iter = arena_ref.iter_slice_range(slice, 1, 3).unwrap();
    assert_eq!(range_iter.next(), Some(&2));
    assert_eq!(range_iter.next(), Some(&4));
    assert_eq!(range_iter.next(), None);
}

#[test]
fn test_arena_ref_clear_and_generation() {
    let mut arena: Arena<512> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    let id = arena_ref.alloc(42u32).unwrap();
    assert_eq!(*arena_ref.get(id).unwrap(), 42);
    
    let gen_before = arena_ref.generation();
    arena_ref.clear();
    
    // Generation should increment
    assert_eq!(arena_ref.generation(), gen_before + 1);
    
    // Old ID should be invalid
    assert!(arena_ref.get(id).is_err());
    
    // Arena should be empty
    assert_eq!(arena_ref.used(), 0);
    assert_eq!(arena_ref.remaining(), 512);
}

#[test]
fn test_arena_ref_restore() {
    let mut arena: Arena<1024> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    let _id1 = arena_ref.alloc(100u32).unwrap();
    let checkpoint = arena_ref.used();
    
    let _id2 = arena_ref.alloc(200u32).unwrap();
    let _id3 = arena_ref.alloc(300u32).unwrap();
    
    assert!(arena_ref.used() > checkpoint);
    
    // Restore to checkpoint
    arena_ref.restore_to(checkpoint);
    
    // Should have rolled back allocations
    assert_eq!(arena_ref.used(), checkpoint);
}

#[test]
fn test_arena_ref_pop() {
    let mut arena: Arena<512> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Pop from empty should fail
    assert!(arena_ref.pop::<u32>().is_err());
    
    // Allocate and pop
    let _id1 = arena_ref.alloc(42u32).unwrap();
    let _id2 = arena_ref.alloc(100u64).unwrap();
    
    // Pop with correct type should succeed
    assert_eq!(arena_ref.pop::<u64>().unwrap(), 100);
    
    // After popping, last_alloc is None, so any pop should fail
    // (pop only tracks the very last allocation, not a stack)
    assert!(arena_ref.pop::<u32>().is_err());
    assert!(arena_ref.pop::<u16>().is_err());
    
    // Allocate again and pop
    let _id3 = arena_ref.alloc(200u32).unwrap();
    assert_eq!(arena_ref.pop::<u32>().unwrap(), 200);
    
    // Now arena should have no tracked last allocation
    assert!(arena_ref.pop::<u32>().is_err());
}

#[test]
fn test_arena_ref_size_erasure() {
    // This is the key feature - we can pass ArenaRef to functions
    // without knowing the arena size at compile time
    
    fn allocate_in_any_arena(arena_ref: &mut ArenaRef) -> ArenaId<u32> {
        arena_ref.alloc(999u32).unwrap()
    }
    
    fn get_from_any_arena(arena_ref: &ArenaRef, id: ArenaId<u32>) -> u32 {
        *arena_ref.get(id).unwrap()
    }
    
    // Test with different sized arenas
    let mut arena1: Arena<256> = Arena::new();
    let mut arena_ref1 = arena1.as_ref();
    
    let mut arena2: Arena<1024> = Arena::new();
    let mut arena_ref2 = arena2.as_ref();
    
    let mut arena3: Arena<4096> = Arena::new();
    let mut arena_ref3 = arena3.as_ref();
    
    // All should work with the same functions
    let id1 = allocate_in_any_arena(&mut arena_ref1);
    let id2 = allocate_in_any_arena(&mut arena_ref2);
    let id3 = allocate_in_any_arena(&mut arena_ref3);
    
    assert_eq!(get_from_any_arena(&arena_ref1, id1), 999);
    assert_eq!(get_from_any_arena(&arena_ref2, id2), 999);
    assert_eq!(get_from_any_arena(&arena_ref3, id3), 999);
}

#[test]
fn test_arena_ref_metadata() {
    let mut arena: Arena<2048> = Arena::new();
    let arena_ref = arena.as_ref();
    
    // Check metadata
    assert_eq!(arena_ref.capacity(), 2048);
    assert_eq!(arena_ref.generation(), 0);
    
    // Arena ID should be non-zero
    assert!(arena_ref.arena_id() > 0);
}

// Text operations are not supported directly through ArenaRef
// since Text methods are hardcoded to use Arena directly

// Buffer operations are not supported directly through ArenaRef
// since Buffer methods are hardcoded to use Arena directly

#[test]
fn test_arena_ref_alignment() {
    let mut arena: Arena<512> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Allocate with different alignments
    let _u8_id = arena_ref.alloc(1u8).unwrap();
    let u64_id = arena_ref.alloc(42u64).unwrap();
    
    // Check that u64 is properly aligned
    let u64_ptr = arena_ref.get(u64_id).unwrap() as *const u64;
    assert_eq!(u64_ptr as usize % core::mem::align_of::<u64>(), 0);
}

#[test]
fn test_arena_ref_generational_safety() {
    let mut arena: Arena<256> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    let id = arena_ref.alloc(42u32).unwrap();
    let gen1 = arena_ref.generation();
    assert_eq!(id.generation(), gen1);
    
    arena_ref.clear();
    let gen2 = arena_ref.generation();
    assert_eq!(gen2, gen1 + 1);
    
    // Old ID should fail due to generation mismatch
    assert!(arena_ref.get(id).is_err());
    
    // New allocation should work
    let new_id = arena_ref.alloc(100u32).unwrap();
    assert_eq!(new_id.generation(), gen2);
    assert_eq!(*arena_ref.get(new_id).unwrap(), 100);
}

#[test]
fn test_arena_ref_custom_index_type() {
    let mut arena: Arena<256, u16> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    let id = arena_ref.alloc(42u32).unwrap();
    assert_eq!(*arena_ref.get(id).unwrap(), 42);
    
    // The ID should use u16 for offset/size
    assert_eq!(id.offset(), 0);
    assert_eq!(id.size(), 4);
}

#[test]
fn test_arena_ref_type_markers() {
    struct MarkerA;
    struct MarkerB;
    
    let mut arena_a: Arena<512, u32, MarkerA> = Arena::new();
    let mut arena_b: Arena<512, u32, MarkerB> = Arena::new();
    
    let mut arena_ref_a = arena_a.as_ref();
    let mut arena_ref_b = arena_b.as_ref();
    
    let id_a = arena_ref_a.alloc(42u32).unwrap();
    let id_b = arena_ref_b.alloc(100u32).unwrap();
    
    // Should work with matching markers
    assert_eq!(*arena_ref_a.get(id_a).unwrap(), 42);
    assert_eq!(*arena_ref_b.get(id_b).unwrap(), 100);
    
    // These would not compile due to type mismatch:
    // arena_ref_a.get(id_b); // Compile error!
    // arena_ref_b.get(id_a); // Compile error!
}

#[test]
fn test_arena_ref_lifetime_safety() {
    let mut arena: Arena<256> = Arena::new();
    
    // Create a scope to test lifetime
    let id = {
        let mut arena_ref = arena.as_ref();
        arena_ref.alloc(42u32).unwrap()
    };
    
    // Can create a new ref and use the ID
    let arena_ref = arena.as_ref();
    assert_eq!(*arena_ref.get(id).unwrap(), 42);
}

#[test]
fn test_arena_ref_stress() {
    let mut arena: Arena<8192> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Perform many allocations through ArenaRef and verify as we go
    let mut last_id = None;
    
    for i in 0..100 {
        let id = arena_ref.alloc(i as u32).unwrap();
        assert_eq!(*arena_ref.get(id).unwrap(), i as u32);
        last_id = Some(id);
    }
    
    // Verify the last allocation is still accessible
    if let Some(id) = last_id {
        assert_eq!(*arena_ref.get(id).unwrap(), 99);
    }
    
    // Create slices
    let slice1 = arena_ref.alloc_slice(&[1, 2, 3, 4, 5]).unwrap();
    let slice2 = arena_ref.alloc_slice_from_fn(10, |i| i * 2).unwrap();
    
    assert_eq!(arena_ref.get_slice(slice1).unwrap().len(), 5);
    assert_eq!(arena_ref.get_slice(slice2).unwrap().len(), 10);
}

#[test]
fn test_arena_ref_zero_sized_types() {
    let mut arena: Arena<256> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Allocate zero-sized type
    let id = arena_ref.alloc(()).unwrap();
    assert_eq!(*arena_ref.get(id).unwrap(), ());
    
    // Should not consume any space
    let used_before = arena_ref.used();
    let _id2 = arena_ref.alloc(()).unwrap();
    assert_eq!(arena_ref.used(), used_before); // No additional space used
}

#[test]
fn test_arena_ref_capacity_limits() {
    let mut arena: Arena<16> = Arena::new(); // Very small arena
    let mut arena_ref = arena.as_ref();
    
    // Should work
    let _id1 = arena_ref.alloc(42u64).unwrap(); // 8 bytes
    let _id2 = arena_ref.alloc(24u64).unwrap(); // 8 bytes, total 16
    
    // Should fail - no more space
    assert!(arena_ref.alloc(100u64).is_err());
}

#[test]
fn test_multiple_arena_refs() {
    let mut arena: Arena<1024> = Arena::new();
    
    // Allocate something with first ref
    let id = {
        let mut arena_ref1 = arena.as_ref();
        arena_ref1.alloc(42u32).unwrap()
    };
    
    // Access with second ref
    {
        let arena_ref2 = arena.as_ref();
        assert_eq!(*arena_ref2.get(id).unwrap(), 42);
    }
    
    // Modify with third ref
    {
        let mut arena_ref3 = arena.as_ref();
        *arena_ref3.get_mut(id).unwrap() = 100;
    }
    
    // Verify with fourth ref
    let arena_ref4 = arena.as_ref();
    assert_eq!(*arena_ref4.get(id).unwrap(), 100);
}