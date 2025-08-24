use super::*;

#[test]
fn test_basic_slice() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice::<u32>(5).unwrap();
    assert_eq!(slice.len(), 5);
    assert!(!slice.is_empty());

    let slice = arena.get_slice(&slice).unwrap();
    assert_eq!(slice.len(), 5);

    // Should all be default values (0)
    for &val in slice {
        assert_eq!(val, 0);
    }
}

#[test]
fn test_slice_with_function() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice_from_fn(5, |i| i as u32 * 10).unwrap();
    let slice = arena.get_slice(&slice).unwrap();

    assert_eq!(slice, &[0, 10, 20, 30, 40]);
}

#[test]
fn test_empty_slice() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice::<u32>(0).unwrap();
    assert_eq!(slice.len(), 0);
    assert!(slice.is_empty());

    let slice = arena.get_slice(&slice).unwrap();
    assert_eq!(slice.len(), 0);
}

#[test]
fn test_slice_mutable_access() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice_from_fn(3, |i| i as u32).unwrap();

    {
        let slice = arena.get_slice_mut(&slice).unwrap();
        slice[1] = 100;
    }

    let slice = arena.get_slice(&slice).unwrap();
    assert_eq!(slice, &[0, 100, 2]);
}

#[test]
fn test_slice_generational_safety() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice::<u32>(3).unwrap();
    let gen1 = arena.generation();
    assert_eq!(slice.generation(), gen1);

    // Should work
    assert!(arena.get_slice(&slice).is_ok());
    assert!(arena.is_slice_valid(&slice));

    // Clear arena
    arena.clear();
    let gen2 = arena.generation();
    assert_eq!(gen2, gen1 + 1);

    // Pool should be invalid
    assert!(arena.get_slice(&slice).is_err());
    assert!(!arena.is_slice_valid(&slice));
}

#[test]
fn test_slice_capacity_info() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice::<u64>(10).unwrap();
    assert_eq!(slice.len(), 10);
    assert_eq!(slice.size_bytes(), 80); // 10 * 8 bytes
    assert_eq!(slice.capacity(), 10);
}

#[test]
fn test_multiple_slices() {
    let mut arena: Arena<1024> = Arena::new();

    let slice1 = arena.alloc_slice_from_fn(3, |i| i as u32).unwrap();
    let slice2 = arena.alloc_slice_from_fn(2, |i| (i + 10) as u32).unwrap();

    let slice1 = arena.get_slice(&slice1).unwrap();
    let slice2 = arena.get_slice(&slice2).unwrap();

    assert_eq!(slice1, &[0, 1, 2]);
    assert_eq!(slice2, &[10, 11]);
}

#[test]
fn test_slice_restore_safety() {
    let mut arena: Arena<1024> = Arena::new();

    let slice1 = arena.alloc_slice::<u32>(3).unwrap();
    let checkpoint = arena.used();
    let slice2 = arena.alloc_slice::<u32>(2).unwrap();

    // Both should work initially
    assert!(arena.get_slice(&slice1).is_ok());
    assert!(arena.get_slice(&slice2).is_ok());

    // Restore to checkpoint
    arena.restore_to(checkpoint);

    // Both should be invalid due to generation change
    assert!(arena.get_slice(&slice1).is_err());
    assert!(arena.get_slice(&slice2).is_err());
}
