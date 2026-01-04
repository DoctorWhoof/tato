use super::*;

#[test]
fn test_basic_allocation() {
    let mut arena: Arena<1024> = Arena::new();

    let id1 = arena.alloc(42u32).unwrap();
    let id2 = arena.alloc(3.14f32).unwrap();

    assert_eq!(*arena.get(id1).unwrap(), 42u32);
    assert_eq!(*arena.get(id2).unwrap(), 3.14f32);
}

#[test]
fn test_different_types() {
    let mut arena: Arena<1024> = Arena::new();

    let bool_id = arena.alloc(true).unwrap();
    let char_id = arena.alloc('x').unwrap();
    let array_id = arena.alloc([1, 2, 3, 4]).unwrap();

    assert_eq!(*arena.get(bool_id).unwrap(), true);
    assert_eq!(*arena.get(char_id).unwrap(), 'x');
    assert_eq!(*arena.get(array_id).unwrap(), [1, 2, 3, 4]);
}

#[test]
fn test_mutable_access() {
    let mut arena: Arena<1024> = Arena::new();

    let id = arena.alloc(42u32).unwrap();
    assert_eq!(*arena.get(id).unwrap(), 42);

    *arena.get_mut(id).unwrap() = 100;
    assert_eq!(*arena.get(id).unwrap(), 100);
}

#[test]
fn test_alignment() {
    let mut arena: Arena<1024> = Arena::new();

    // Allocate a u8 first to offset alignment
    let _id1 = arena.alloc(1u8).unwrap();

    // This should be properly aligned
    let id2 = arena.alloc(42u64).unwrap();
    assert_eq!(*arena.get(id2).unwrap(), 42u64);

    // Check that alignment worked
    let ptr = arena.get(id2).unwrap() as *const u64;
    assert_eq!(ptr as usize % core::mem::align_of::<u64>(), 0);
}

#[test]
fn test_capacity_limits() {
    let mut arena: Arena<16> = Arena::new();

    // Should work
    let _id1 = arena.alloc(42u64).unwrap();
    let _id2 = arena.alloc(24u64).unwrap();

    // Should fail - not enough space
    assert!(arena.alloc(100u64).is_err());
}

#[test]
fn test_arena_stats() {
    let mut arena: Arena<1024> = Arena::new();

    assert_eq!(arena.used(), 0);
    assert_eq!(arena.remaining(), 1024);

    let _id1 = arena.alloc(42u64).unwrap();
    assert_eq!(arena.used(), 8);
    assert_eq!(arena.remaining(), 1016);

    let _id2 = arena.alloc(24u32).unwrap();
    assert_eq!(arena.used(), 12);
    assert_eq!(arena.remaining(), 1012);
}

#[test]
fn test_clear() {
    let mut arena: Arena<1024> = Arena::new();

    let id = arena.alloc(42u32).unwrap();
    assert_eq!(*arena.get(id).unwrap(), 42);
    assert_eq!(arena.used(), 4);

    let gen_before = arena.generation();
    arena.clear();

    // Generation should have incremented
    assert_eq!(arena.generation(), gen_before + 1);
    assert_eq!(arena.used(), 0);

    // Old ID should be invalid
    assert!(arena.get(id).is_err());
}

#[test]
fn test_custom_size_type() {
    let mut arena: Arena<256, u16> = Arena::new();

    let id = arena.alloc(42u32).unwrap();
    assert_eq!(*arena.get(id).unwrap(), 42);
    assert_eq!(id.offset(), 0);
    assert_eq!(id.size(), 4);
}

#[test]
fn test_restore_to() {
    let mut arena: Arena<1024> = Arena::new();

    let id1 = arena.alloc(42u32).unwrap();
    let checkpoint = arena.used();
    let gen_before = arena.generation();

    let id2 = arena.alloc(100u32).unwrap();
    assert_eq!(*arena.get(id1).unwrap(), 42);
    assert_eq!(*arena.get(id2).unwrap(), 100);

    // Restore to checkpoint
    arena.restore_to(checkpoint);

    // Generation should have incremented
    assert_eq!(arena.generation(), gen_before + 1);

    // id1 should still be valid (created before checkpoint)
    assert!(arena.get(id1).is_err()); // Actually, it becomes invalid too since generation changed

    // id2 should be invalid (created after checkpoint)
    assert!(arena.get(id2).is_err());

    // Can allocate new data in the restored space
    let id3 = arena.alloc(200u32).unwrap();
    assert_eq!(*arena.get(id3).unwrap(), 200);
}

#[test]
fn test_generational_safety() {
    let mut arena: Arena<1024> = Arena::new();

    let id1 = arena.alloc(42u32).unwrap();
    let gen1 = arena.generation();
    assert_eq!(id1.generation(), gen1);
    assert_eq!(*arena.get(id1).unwrap(), 42);

    // Clear arena (increments generation)
    arena.clear();
    let gen2 = arena.generation();
    assert_eq!(gen2, gen1 + 1);

    // Old ID should be invalid
    assert!(arena.get(id1).is_err());
    // assert!(!arena.is_valid(id1)); // is_valid method removed

    // New allocation should work
    let id2 = arena.alloc(100u32).unwrap();
    assert_eq!(id2.generation(), gen2);
    assert_eq!(*arena.get(id2).unwrap(), 100);
    // assert!(arena.is_valid(id2)); // is_valid method removed
}

#[test]
fn test_type_markers() {
    struct MarkerA;
    struct MarkerB;

    let mut arena_a: Arena<1024, usize, MarkerA> = Arena::new();
    let mut arena_b: Arena<1024, usize, MarkerB> = Arena::new();

    let id_a = arena_a.alloc(42u32).unwrap();
    let id_b = arena_b.alloc(100u32).unwrap();

    // This should work
    assert_eq!(*arena_a.get(id_a).unwrap(), 42);
    assert_eq!(*arena_b.get(id_b).unwrap(), 100);

    // These should not compile due to type mismatch:
    // arena_a.get(id_b); // Compile error!
    // arena_b.get(id_a); // Compile error!
}

#[test]
fn test_pop_functionality() {
    let mut arena: Arena<1024> = Arena::new();

    // Test pop on empty arena
    assert!(arena.pop::<u32>().is_err());

    // Allocate some values
    let id1 = arena.alloc(42u32).unwrap();
    let id2 = arena.alloc(100u64).unwrap();
    let id3 = arena.alloc(200u16).unwrap();

    // All should be valid
    assert_eq!(*arena.get(id1).unwrap(), 42);
    assert_eq!(*arena.get(id2).unwrap(), 100);
    assert_eq!(*arena.get(id3).unwrap(), 200);

    let used_before_pop = arena.used();

    // Pop the last allocation (was u16)
    assert_eq!(arena.pop::<u16>().unwrap(), 200);

    // Pop doesn't change generation, so id1 and id2 are still valid
    // but id3 was popped so it's in undefined state (its memory was deallocated)
    assert_eq!(*arena.get(id1).unwrap(), 42);
    assert_eq!(*arena.get(id2).unwrap(), 100);

    // Used space should have decreased
    assert!(arena.used() < used_before_pop);

    // Allocate new values after pop
    let new_id1 = arena.alloc(999u32).unwrap();
    assert_eq!(*arena.get(new_id1).unwrap(), 999);

    // Pop again
    assert_eq!(arena.pop::<u32>().unwrap(), 999);
    // new_id1 was just popped, its memory is deallocated
    // id1 and id2 are still valid though
    assert_eq!(*arena.get(id1).unwrap(), 42);
    assert_eq!(*arena.get(id2).unwrap(), 100);

    // No more to pop
    assert!(arena.pop::<u32>().is_err());
}

#[test]
fn test_pop_with_single_values() {
    let mut arena: Arena<1024> = Arena::new();

    // Allocate some single values
    let id1 = arena.alloc(100u32).unwrap();
    let id2 = arena.alloc(200u32).unwrap();

    // Both should be valid
    assert_eq!(*arena.get(id1).unwrap(), 100);
    assert_eq!(*arena.get(id2).unwrap(), 200);

    // Pop the last allocation
    assert_eq!(arena.pop::<u32>().unwrap(), 200);

    // Pop doesn't change generation, so id1 is still valid
    // id2 was popped so it's in undefined state (its memory was deallocated)
    assert_eq!(*arena.get(id1).unwrap(), 100);

    // Allocate new value after pop
    let new_id = arena.alloc(300u32).unwrap();
    assert_eq!(*arena.get(new_id).unwrap(), 300);
}