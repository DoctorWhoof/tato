use super::*;

#[test]
fn test_zero_size_arena() {
    let mut arena: Arena<0> = Arena::new();

    // Should fail immediately
    assert!(arena.alloc(42u32).is_err());
    assert!(arena.alloc_slice::<u32>(1).is_err());
}

#[test]
fn test_small_size_type() {
    let mut arena: Arena<100, u8> = Arena::new();

    let id = arena.alloc(42u32).unwrap();
    assert_eq!(*arena.get(&id).unwrap(), 42);

    // Should work up to 255 bytes
    assert!(arena.used() <= 255);
}

#[test]
fn test_arena_full() {
    let mut arena: Arena<8> = Arena::new(); // Very small arena

    let id1 = arena.alloc(42u32).unwrap(); // 4 bytes
    let id2 = arena.alloc(24u32).unwrap(); // 4 bytes, total 8

    assert_eq!(*arena.get(&id1).unwrap(), 42);
    assert_eq!(*arena.get(&id2).unwrap(), 24);

    // Should be full now
    assert!(arena.alloc(1u8).is_err());
}

#[test]
fn test_invalid_restore_offset() {
    let mut arena: Arena<1024> = Arena::new();

    let gen_before = arena.generation();

    // Restore to invalid offset (beyond arena size) should not panic
    // but should not change anything
    arena.restore_to(2000);

    // Generation should NOT increment for invalid offsets
    assert_eq!(arena.generation(), gen_before);
}
