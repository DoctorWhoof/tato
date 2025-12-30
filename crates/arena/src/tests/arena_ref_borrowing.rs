use super::*;

#[test]
fn test_arena_ref_borrows_arena_mutably() {
    let mut arena: Arena<1024> = Arena::new();
    
    // Allocate something before creating ArenaRef
    let id_before = arena.alloc(42u32).unwrap();
    assert_eq!(*arena.get(id_before).unwrap(), 42);
    
    // Create ArenaRef - this takes &mut arena
    let mut arena_ref = arena.as_ref();
    
    // The following would NOT compile because arena is mutably borrowed:
    // arena.alloc(100u32); // ERROR: cannot borrow `arena` as mutable because it is also borrowed as mutable
    // arena.get(id_before); // ERROR: cannot borrow `arena` as immutable because it is also borrowed as mutable
    
    // But we CAN use the ArenaRef
    let id_during = arena_ref.alloc(100u32).unwrap();
    assert_eq!(*arena_ref.get(id_during).unwrap(), 100);
    
    // And the old ID is still valid through ArenaRef
    assert_eq!(*arena_ref.get(id_before).unwrap(), 42);
}

#[test]
fn test_arena_ref_does_not_invalidate_existing_ids() {
    let mut arena: Arena<1024> = Arena::new();
    
    // Create some allocations
    let id1 = arena.alloc(1u32).unwrap();
    let id2 = arena.alloc(2u32).unwrap();
    let slice = arena.alloc_slice(&[10, 20, 30]).unwrap();
    
    let gen_before = arena.generation();
    
    // Create ArenaRef
    let arena_ref = arena.as_ref();
    
    // Generation should NOT change when creating ArenaRef
    assert_eq!(arena_ref.generation(), gen_before);
    
    // All existing IDs should still be valid
    assert_eq!(*arena_ref.get(id1).unwrap(), 1);
    assert_eq!(*arena_ref.get(id2).unwrap(), 2);
    assert_eq!(arena_ref.get_slice(slice).unwrap(), &[10, 20, 30]);
}

#[test]
fn test_arena_ref_lifetime_prevents_use_after_drop() {
    let mut arena: Arena<1024> = Arena::new();
    let id = arena.alloc(42u32).unwrap();
    
    {
        let arena_ref = arena.as_ref();
        // ArenaRef is valid here
        assert_eq!(*arena_ref.get(id).unwrap(), 42);
        
        // ArenaRef is dropped at end of scope
    }
    
    // After ArenaRef is dropped, we can use arena again
    assert_eq!(*arena.get(id).unwrap(), 42);
    let new_id = arena.alloc(100u32).unwrap();
    assert_eq!(*arena.get(new_id).unwrap(), 100);
}

#[test]
fn test_cannot_create_multiple_arena_refs_simultaneously() {
    let mut arena: Arena<1024> = Arena::new();
    
    let _arena_ref1 = arena.as_ref();
    
    // The following would NOT compile:
    // let _arena_ref2 = arena.as_ref(); // ERROR: cannot borrow `arena` as mutable more than once at a time
    
    // This test passes by compiling - Rust's borrow checker prevents multiple mutable borrows
}

#[test]
fn test_arena_ref_sequential_creation() {
    let mut arena: Arena<1024> = Arena::new();
    let id1 = arena.alloc(1u32).unwrap();
    
    // First ArenaRef
    {
        let mut arena_ref = arena.as_ref();
        let id2 = arena_ref.alloc(2u32).unwrap();
        assert_eq!(*arena_ref.get(id1).unwrap(), 1);
        assert_eq!(*arena_ref.get(id2).unwrap(), 2);
    }
    
    // Second ArenaRef (after first is dropped)
    {
        let mut arena_ref = arena.as_ref();
        let id3 = arena_ref.alloc(3u32).unwrap();
        // All previous allocations are still valid
        assert_eq!(*arena_ref.get(id1).unwrap(), 1);
        // Note: id2 is not available here as a variable, but the allocation still exists
        assert_eq!(*arena_ref.get(id3).unwrap(), 3);
    }
    
    // Back to using arena directly
    let id4 = arena.alloc(4u32).unwrap();
    assert_eq!(*arena.get(id1).unwrap(), 1);
    assert_eq!(*arena.get(id4).unwrap(), 4);
}

#[test]
fn test_arena_ref_modifications_persist() {
    let mut arena: Arena<512> = Arena::new();
    
    let id = arena.alloc(42u32).unwrap();
    let initial_used = arena.used();
    
    // Modify through ArenaRef
    {
        let mut arena_ref = arena.as_ref();
        
        // Modify existing value
        *arena_ref.get_mut(id).unwrap() = 100;
        
        // Add new allocations
        let _new_id = arena_ref.alloc(200u32).unwrap();
        
        assert!(arena_ref.used() > initial_used);
    }
    
    // Changes persist after ArenaRef is dropped
    assert_eq!(*arena.get(id).unwrap(), 100);
    assert!(arena.used() > initial_used);
}

#[test]
fn test_arena_ref_clear_affects_arena() {
    let mut arena: Arena<256> = Arena::new();
    
    let id = arena.alloc(42u32).unwrap();
    let gen_before = arena.generation();
    
    // Clear through ArenaRef
    {
        let mut arena_ref = arena.as_ref();
        arena_ref.clear();
    }
    
    // Clear should have affected the underlying arena
    assert_eq!(arena.generation(), gen_before + 1);
    assert_eq!(arena.used(), 0);
    assert!(arena.get(id).is_err()); // Old ID is invalid
}

#[test]
fn test_arena_ref_is_just_pointers() {
    // ArenaRef doesn't copy or move the arena data,
    // it just holds pointers to the arena's fields
    
    let mut arena: Arena<1024> = Arena::new();
    
    // Fill arena with some data
    for i in 0..10 {
        arena.alloc(i * 100).unwrap();
    }
    
    let arena_ptr = &arena as *const Arena<1024>;
    let used_before = arena.used();
    
    {
        let arena_ref = arena.as_ref();
        
        // ArenaRef is using the same underlying storage
        assert_eq!(arena_ref.used(), used_before);
        
        // The arena hasn't moved
        assert_eq!(arena_ptr, &arena as *const Arena<1024>);
    }
    
    // Arena is still at the same location and has the same data
    assert_eq!(arena_ptr, &arena as *const Arena<1024>);
    assert_eq!(arena.used(), used_before);
}

#[test]
fn test_arena_ref_respects_marker_types() {
    struct MarkerA;
    struct MarkerB;
    
    let mut arena_a: Arena<512, u32, MarkerA> = Arena::new();
    let mut arena_b: Arena<512, u32, MarkerB> = Arena::new();
    
    let id_a = arena_a.alloc(42u32).unwrap();
    let id_b = arena_b.alloc(100u32).unwrap();
    
    {
        let arena_ref_a = arena_a.as_ref();
        let arena_ref_b = arena_b.as_ref();
        
        // These work
        assert_eq!(*arena_ref_a.get(id_a).unwrap(), 42);
        assert_eq!(*arena_ref_b.get(id_b).unwrap(), 100);
        
        // These would NOT compile (marker type mismatch):
        // arena_ref_a.get(id_b); // ERROR: mismatched types
        // arena_ref_b.get(id_a); // ERROR: mismatched types
    }
}

#[test]
fn test_arena_ref_size_erasure_benefit() {
    // This demonstrates the key benefit: we can write functions
    // that work with any arena size without knowing it at compile time
    
    fn allocate_and_sum<M>(arena_ref: &mut ArenaRef<'_, u32, M>) -> u32 {
        let mut sum = 0u32;
        for i in 1..=5 {
            let id = arena_ref.alloc(i).unwrap();
            sum += *arena_ref.get(id).unwrap();
        }
        sum
    }
    
    // Works with any size arena
    let mut arena_small: Arena<128> = Arena::new();
    let mut arena_medium: Arena<512> = Arena::new();
    let mut arena_large: Arena<4096> = Arena::new();
    
    assert_eq!(allocate_and_sum(&mut arena_small.as_ref()), 15);
    assert_eq!(allocate_and_sum(&mut arena_medium.as_ref()), 15);
    assert_eq!(allocate_and_sum(&mut arena_large.as_ref()), 15);
}

#[test]
fn test_arena_ref_pop_behavior() {
    let mut arena: Arena<256> = Arena::new();
    
    // Allocate through arena
    let id1 = arena.alloc(10u32).unwrap();
    let _id2 = arena.alloc(20u32).unwrap();
    
    {
        let mut arena_ref = arena.as_ref();
        
        // Pop through ArenaRef
        assert_eq!(arena_ref.pop::<u32>().unwrap(), 20);
        
        // id1 is still valid (pop doesn't invalidate other allocations)
        assert_eq!(*arena_ref.get(id1).unwrap(), 10);
        
        // Allocate more through ArenaRef
        let _id3 = arena_ref.alloc(30u32).unwrap();
    }
    
    // After ArenaRef is dropped, arena reflects the changes
    assert_eq!(*arena.get(id1).unwrap(), 10);
    
    // Can pop the last allocation made through ArenaRef
    assert_eq!(arena.pop::<u32>().unwrap(), 30);
}

#[test]
fn test_arena_ref_restore_behavior() {
    let mut arena: Arena<512> = Arena::new();
    
    let _id1 = arena.alloc(1u32).unwrap();
    let checkpoint = arena.used();
    let _id2 = arena.alloc(2u32).unwrap();
    
    let gen_before = arena.generation();
    
    {
        let mut arena_ref = arena.as_ref();
        
        // Add more through ArenaRef
        let _id3 = arena_ref.alloc(3u32).unwrap();
        
        // Restore through ArenaRef
        arena_ref.restore_to(checkpoint);
        
        // Generation should have incremented
        assert_eq!(arena_ref.generation(), gen_before + 1);
        assert_eq!(arena_ref.used(), checkpoint);
    }
    
    // Changes persist in arena
    assert_eq!(arena.generation(), gen_before + 1);
    assert_eq!(arena.used(), checkpoint);
}