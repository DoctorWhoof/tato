use super::*;

/// Tests for basic arena functionality
mod arena_tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut arena: Arena<1024> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let id2 = arena.alloc(3.14f32).unwrap();

        assert_eq!(*arena.get(&id1).unwrap(), 42u32);
        assert_eq!(*arena.get(&id2).unwrap(), 3.14f32);
    }

    #[test]
    fn test_different_types() {
        let mut arena: Arena<1024> = Arena::new();

        let bool_id = arena.alloc(true).unwrap();
        let char_id = arena.alloc('x').unwrap();
        let array_id = arena.alloc([1, 2, 3, 4]).unwrap();

        assert_eq!(*arena.get(&bool_id).unwrap(), true);
        assert_eq!(*arena.get(&char_id).unwrap(), 'x');
        assert_eq!(*arena.get(&array_id).unwrap(), [1, 2, 3, 4]);
    }

    #[test]
    fn test_mutable_access() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        assert_eq!(*arena.get(&id).unwrap(), 42);

        *arena.get_mut(&id).unwrap() = 100;
        assert_eq!(*arena.get(&id).unwrap(), 100);
    }

    #[test]
    fn test_alignment() {
        let mut arena: Arena<1024> = Arena::new();

        // Allocate a u8 first to offset alignment
        let _id1 = arena.alloc(1u8).unwrap();

        // This should be properly aligned
        let id2 = arena.alloc(42u64).unwrap();
        assert_eq!(*arena.get(&id2).unwrap(), 42u64);

        // Check that alignment worked
        let ptr = arena.get(&id2).unwrap() as *const u64;
        assert_eq!(ptr as usize % core::mem::align_of::<u64>(), 0);
    }

    #[test]
    fn test_capacity_limits() {
        let mut arena: Arena<16> = Arena::new();

        // Should work
        let _id1 = arena.alloc(42u64).unwrap();
        let _id2 = arena.alloc(24u64).unwrap();

        // Should fail - not enough space
        assert!(arena.alloc(100u64).is_none());
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
        assert_eq!(*arena.get(&id).unwrap(), 42);
        assert_eq!(arena.used(), 4);

        let gen_before = arena.generation();
        arena.clear();

        // Generation should have incremented
        assert_eq!(arena.generation(), gen_before + 1);
        assert_eq!(arena.used(), 0);

        // Old ID should be invalid
        assert!(arena.get(&id).is_none());
    }

    #[test]
    fn test_custom_size_type() {
        let mut arena: Arena<256, u16> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        assert_eq!(*arena.get(&id).unwrap(), 42);
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
        assert_eq!(*arena.get(&id1).unwrap(), 42);
        assert_eq!(*arena.get(&id2).unwrap(), 100);

        // Restore to checkpoint
        arena.restore_to(checkpoint);

        // Generation should have incremented
        assert_eq!(arena.generation(), gen_before + 1);

        // id1 should still be valid (created before checkpoint)
        assert!(arena.get(&id1).is_none()); // Actually, it becomes invalid too since generation changed

        // id2 should be invalid (created after checkpoint)
        assert!(arena.get(&id2).is_none());

        // Can allocate new data in the restored space
        let id3 = arena.alloc(200u32).unwrap();
        assert_eq!(*arena.get(&id3).unwrap(), 200);
    }

    #[test]
    fn test_generational_safety() {
        let mut arena: Arena<1024> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let gen1 = arena.generation();
        assert_eq!(id1.generation(), gen1);
        assert_eq!(*arena.get(&id1).unwrap(), 42);

        // Clear arena (increments generation)
        arena.clear();
        let gen2 = arena.generation();
        assert_eq!(gen2, gen1 + 1);

        // Old ID should be invalid
        assert!(arena.get(&id1).is_none());
        assert!(!arena.is_valid(&id1));

        // New allocation should work
        let id2 = arena.alloc(100u32).unwrap();
        assert_eq!(id2.generation(), gen2);
        assert_eq!(*arena.get(&id2).unwrap(), 100);
        assert!(arena.is_valid(&id2));
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
        assert_eq!(*arena_a.get(&id_a).unwrap(), 42);
        assert_eq!(*arena_b.get(&id_b).unwrap(), 100);

        // These should not compile due to type mismatch:
        // arena_a.get(&id_b); // Compile error!
        // arena_b.get(&id_a); // Compile error!
    }
}

/// Tests for pool functionality
mod pool_tests {
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

        // Slice should be invalid
        assert!(arena.get_pool(&pool).is_none());
        assert!(!arena.is_pool_valid(&pool));
    }

    #[test]
    fn test_pool_capacity_info() {
        let mut arena: Arena<1024> = Arena::new();

        let pool = arena.alloc_pool::<u64>(10).unwrap();
        assert_eq!(pool.len(), 10);
        assert_eq!(pool.size_bytes(), 80); // 10 * 8 bytes
        assert_eq!(pool.capacity(), (10, 10));
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
}

mod text_tests {
    use super::*;

    #[test]
    fn test_text_from_str() {
        let mut arena: Arena<1024> = Arena::new();

        let text = Text::from_str(&mut arena, "Hello, World!").unwrap();
        assert_eq!(text.len(), 13);

        let s = text.as_str(&arena).unwrap();
        assert_eq!(s, "Hello, World!");
    }

    #[test]
    fn test_text_format() {
        let mut arena: Arena<1024> = Arena::new();

        let text = Text::format(&mut arena, "greeting:", "Hello").unwrap();
        let s = text.as_str(&arena).unwrap();
        assert_eq!(s, "greeting:\"Hello\""); // Strings get formatted with quotes
    }
}

/// Tests for RawId functionality
mod raw_id_tests {
    use super::*;

    #[test]
    fn test_raw_id_conversion() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        let raw = id.raw();
        let typed_back: ArenaId<u32, u16, ()> = raw.typed();

        assert_eq!(id.offset(), typed_back.offset());
        assert_eq!(id.size(), typed_back.size());
        assert_eq!(id.generation(), typed_back.generation());
    }

    #[test]
    fn test_raw_id_generation() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        let raw = id.raw();

        assert_eq!(raw.generation(), arena.generation());
        assert_eq!(raw.generation(), id.generation());
    }

    #[test]
    fn test_raw_id_type_safety() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u64).unwrap(); // 8 bytes
        let raw = id.raw();

        // This should work (correct size)
        let _typed_u64: ArenaId<u64, u16, ()> = raw.typed();

        // This should panic in debug mode (wrong size)
        // let _typed_u32: ArenaId<u32, usize, ()> = raw.typed(); // Would panic
    }
}

/// Tests for unsafe unchecked access
mod unsafe_tests {
    use super::*;

    #[test]
    fn test_unchecked_access() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();

        // Safe access
        assert_eq!(*arena.get(&id).unwrap(), 42);

        // Unsafe unchecked access (should be same result)
        unsafe {
            assert_eq!(*arena.get_unchecked(&id), 42);
        }
    }

    #[test]
    fn test_unchecked_pool_access() {
        let mut arena: Arena<1024> = Arena::new();

        let pool = arena.alloc_pool_from_fn(3, |i| i as u32).unwrap();

        // Safe access
        let safe_slice = arena.get_pool(&pool).unwrap();
        assert_eq!(safe_slice, &[0, 1, 2]);

        // Unsafe unchecked access
        unsafe {
            let unsafe_slice = arena.get_pool_unchecked(&pool);
            assert_eq!(unsafe_slice, &[0, 1, 2]);
        }
    }

    #[test]
    fn test_unchecked_mutable_access() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();

        unsafe {
            *arena.get_unchecked_mut(&id) = 100;
        }

        assert_eq!(*arena.get(&id).unwrap(), 100);
    }
}

/// Tests for edge cases and error conditions
mod edge_cases {
    use super::*;

    #[test]
    fn test_zero_size_arena() {
        let mut arena: Arena<0> = Arena::new();

        // Should fail immediately
        assert!(arena.alloc(42u32).is_none());
        assert!(arena.alloc_pool::<u32>(1).is_none());
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
        assert!(arena.alloc(1u8).is_none());
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
}
