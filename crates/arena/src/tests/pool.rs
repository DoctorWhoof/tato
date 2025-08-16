use super::*;

#[test]
fn test_basic_pool() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool::<u32>(5).unwrap();
    assert_eq!(pool.len(), 5);
    assert!(!pool.is_empty());

    let slice = arena.get_pool(&pool).unwrap();
    assert_eq!(slice.len(), 5);

    // Should all be default values (0)
    for &val in slice {
        assert_eq!(val, 0);
    }
}

#[test]
fn test_pool_with_function() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool_from_fn(5, |i| i as u32 * 10).unwrap();
    let slice = arena.get_pool(&pool).unwrap();

    assert_eq!(slice, &[0, 10, 20, 30, 40]);
}

#[test]
fn test_empty_pool() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool::<u32>(0).unwrap();
    assert_eq!(pool.len(), 0);
    assert!(pool.is_empty());

    let slice = arena.get_pool(&pool).unwrap();
    assert_eq!(slice.len(), 0);
}

#[test]
fn test_pool_mutable_access() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool_from_fn(3, |i| i as u32).unwrap();

    {
        let slice = arena.get_pool_mut(&pool).unwrap();
        slice[1] = 100;
    }

    let slice = arena.get_pool(&pool).unwrap();
    assert_eq!(slice, &[0, 100, 2]);
}

#[test]
fn test_pool_generational_safety() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool::<u32>(3).unwrap();
    let gen1 = arena.generation();
    assert_eq!(pool.generation(), gen1);

    // Should work
    assert!(arena.get_pool(&pool).is_some());
    assert!(arena.is_pool_valid(&pool));

    // Clear arena
    arena.clear();
    let gen2 = arena.generation();
    assert_eq!(gen2, gen1 + 1);

    // Pool should be invalid
    assert!(arena.get_pool(&pool).is_none());
    assert!(!arena.is_pool_valid(&pool));
}

#[test]
fn test_pool_capacity_info() {
    let mut arena: Arena<1024> = Arena::new();

    let pool = arena.alloc_pool::<u64>(10).unwrap();
    assert_eq!(pool.len(), 10);
    assert_eq!(pool.size_bytes(), 80); // 10 * 8 bytes
    assert_eq!(pool.capacity(), 10);
}

#[test]
fn test_multiple_pools() {
    let mut arena: Arena<1024> = Arena::new();

    let pool1 = arena.alloc_pool_from_fn(3, |i| i as u32).unwrap();
    let pool2 = arena.alloc_pool_from_fn(2, |i| (i + 10) as u32).unwrap();

    let slice1 = arena.get_pool(&pool1).unwrap();
    let slice2 = arena.get_pool(&pool2).unwrap();

    assert_eq!(slice1, &[0, 1, 2]);
    assert_eq!(slice2, &[10, 11]);
}

#[test]
fn test_pool_restore_safety() {
    let mut arena: Arena<1024> = Arena::new();

    let pool1 = arena.alloc_pool::<u32>(3).unwrap();
    let checkpoint = arena.used();
    let pool2 = arena.alloc_pool::<u32>(2).unwrap();

    // Both should work initially
    assert!(arena.get_pool(&pool1).is_some());
    assert!(arena.get_pool(&pool2).is_some());

    // Restore to checkpoint
    arena.restore_to(checkpoint);

    // Both should be invalid due to generation change
    assert!(arena.get_pool(&pool1).is_none());
    assert!(arena.get_pool(&pool2).is_none());
}
