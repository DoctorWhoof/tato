//! Comprehensive tests for the TatoArena crate
//!
//! This module contains all tests for the arena allocator, pools, and IDs.
//! Tests are organized by functionality and include both unit tests and
//! integration tests.

use super::*;


/// Tests for basic arena functionality
mod arena_tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut arena: Arena<1024> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let id2 = arena.alloc(3.14f32).unwrap();

        assert_eq!(*arena.get(&id1), 42u32);
        assert_eq!(*arena.get(&id2), 3.14f32);
    }

    #[test]
    fn test_different_types() {
        let mut arena: Arena<1024> = Arena::new();

        let bool_id = arena.alloc(true).unwrap();
        let char_id = arena.alloc('x').unwrap();
        let array_id = arena.alloc([1, 2, 3, 4]).unwrap();

        assert_eq!(*arena.get(&bool_id), true);
        assert_eq!(*arena.get(&char_id), 'x');
        assert_eq!(*arena.get(&array_id), [1, 2, 3, 4]);
    }

    #[test]
    fn test_mutable_access() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        assert_eq!(*arena.get(&id), 42);

        *arena.get_mut(&id) = 100;
        assert_eq!(*arena.get(&id), 100);
    }

    #[test]
    fn test_alignment() {
        let mut arena: Arena<1024> = Arena::new();

        // Allocate a u8 first to offset alignment
        let _id1 = arena.alloc(1u8).unwrap();

        // This should be properly aligned
        let id2 = arena.alloc(42u32).unwrap();

        // Check that the u32 is aligned properly
        assert_eq!(id2.offset() % align_of::<u32>(), 0);
        assert_eq!(*arena.get(&id2), 42u32);
    }

    #[test]
    fn test_mixed_alignment() {
        let mut arena: Arena<1024> = Arena::new();

        let u8_id = arena.alloc(1u8).unwrap();
        let u64_id = arena.alloc(0x123456789ABCDEFu64).unwrap();
        let u16_id = arena.alloc(0x1234u16).unwrap();

        // Check alignments
        assert_eq!(u8_id.offset() % align_of::<u8>(), 0);
        assert_eq!(u64_id.offset() % align_of::<u64>(), 0);
        assert_eq!(u16_id.offset() % align_of::<u16>(), 0);

        // Check values
        assert_eq!(*arena.get(&u8_id), 1u8);
        assert_eq!(*arena.get(&u64_id), 0x123456789ABCDEFu64);
        assert_eq!(*arena.get(&u16_id), 0x1234u16);
    }

    #[test]
    fn test_out_of_memory() {
        let mut arena: Arena<7> = Arena::new();

        // This should succeed (4 bytes)
        let _id1 = arena.alloc(42u32).unwrap();

        // This should fail - not enough space (only 3 bytes left)
        let result = arena.alloc(99u32);
        assert!(result.is_none());
    }

    #[test]
    fn test_exact_fit() {
        let mut arena: Arena<8> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let id2 = arena.alloc(99u32).unwrap();

        assert_eq!(*arena.get(&id1), 42);
        assert_eq!(*arena.get(&id2), 99);

        // Should be no space left
        let result = arena.alloc(1u8);
        assert!(result.is_none());
    }

    #[test]
    fn test_clear() {
        let mut arena: Arena<1024> = Arena::new();

        let _id1 = arena.alloc(42u32).unwrap();
        let _id2 = arena.alloc(3.14f32).unwrap();

        assert_eq!(arena.used(), 8);
        assert_eq!(arena.allocation_count(), 2);

        arena.clear();
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.allocation_count(), 0);
        assert_eq!(arena.remaining(), 1024);
    }

    #[test]
    fn test_arena_stats() {
        let mut arena: Arena<1024> = Arena::new();

        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 1024);
        assert_eq!(arena.allocation_count(), 0);

        let _id1 = arena.alloc(42u32).unwrap();
        assert_eq!(arena.used(), 4);
        assert_eq!(arena.remaining(), 1020);
        assert_eq!(arena.allocation_count(), 1);

        let _id2 = arena.alloc(3.14f64).unwrap();
        assert_eq!(arena.used(), 16); // 4 + 4 padding + 8
        assert_eq!(arena.remaining(), 1008);
        assert_eq!(arena.allocation_count(), 2);
    }
}

/// Tests for pool functionality
mod pool_tests {
    use super::*;

    #[test]
    fn test_basic_pool() {
        let mut arena: Arena<1024> = Arena::new();

        let pool_id = arena.alloc_pool::<u32>(5).unwrap();
        assert_eq!(pool_id.len(), 5);
        assert!(!pool_id.is_empty());

        // Get mutable slice and set values
        let slice = arena.get_pool_mut(&pool_id);
        slice[0] = 10;
        slice[1] = 20;
        slice[2] = 30;
        slice[3] = 40;
        slice[4] = 50;

        // Read back values
        let slice = arena.get_pool(&pool_id);
        assert_eq!(slice, &[10, 20, 30, 40, 50]);
    }

    #[test]
    fn test_empty_pool() {
        let mut arena: Arena<1024> = Arena::new();

        let pool_id = arena.alloc_pool::<u32>(0).unwrap();
        assert_eq!(pool_id.len(), 0);
        assert!(pool_id.is_empty());

        // Should return empty slice
        let slice = arena.get_pool(&pool_id);
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn test_pool_default_initialization() {
        let mut arena: Arena<1024> = Arena::new();

        let pool_id = arena.alloc_pool::<u32>(3).unwrap();
        let slice = arena.get_pool(&pool_id);

        // Should be initialized with Default::default()
        assert_eq!(slice, &[0, 0, 0]);
    }

    #[test]
    fn test_pool_out_of_memory() {
        let mut arena: Arena<16> = Arena::new();

        // This should fail - trying to allocate 10 u32s (40 bytes) in 16 bytes
        let result = arena.alloc_pool::<u32>(10);
        assert!(result.is_none());
    }

    #[test]
    fn test_multiple_pools() {
        let mut arena: Arena<1024> = Arena::new();

        let pool1 = arena.alloc_pool::<u8>(10).unwrap();
        let pool2 = arena.alloc_pool::<u16>(5).unwrap();
        let pool3 = arena.alloc_pool::<u32>(3).unwrap();

        assert_eq!(pool1.len(), 10);
        assert_eq!(pool2.len(), 5);
        assert_eq!(pool3.len(), 3);

        // Test they don't interfere with each other
        let slice1 = arena.get_pool_mut(&pool1);
        slice1[0] = 100;

        let slice2 = arena.get_pool_mut(&pool2);
        slice2[0] = 200;

        let slice3 = arena.get_pool_mut(&pool3);
        slice3[0] = 300;

        assert_eq!(arena.get_pool(&pool1)[0], 100);
        assert_eq!(arena.get_pool(&pool2)[0], 200);
        assert_eq!(arena.get_pool(&pool3)[0], 300);
    }

    #[test]
    fn test_pool_alignment() {
        let mut arena: Arena<1024> = Arena::new();

        // Allocate a u8 to offset alignment
        let _single = arena.alloc(1u8).unwrap();

        // Pool should be properly aligned
        let pool = arena.alloc_pool::<u64>(2).unwrap();
        assert_eq!(pool.offset() % align_of::<u64>(), 0);

        let slice = arena.get_pool_mut(&pool);
        slice[0] = 0x123456789ABCDEF0;
        slice[1] = 0x0FEDCBA987654321;

        let slice = arena.get_pool(&pool);
        assert_eq!(slice[0], 0x123456789ABCDEF0);
        assert_eq!(slice[1], 0x0FEDCBA987654321);
    }

    #[test]
    fn test_pool_slice_operations() {
        let mut arena: Arena<1024> = Arena::new();

        let pool = arena.alloc_pool::<i32>(10).unwrap();
        let slice = arena.get_pool_mut(&pool);

        // Fill with pattern
        for (i, val) in slice.iter_mut().enumerate() {
            *val = i as i32 * 2;
        }

        let slice = arena.get_pool(&pool);
        assert_eq!(slice.len(), 10);
        assert_eq!(slice.first(), Some(&0));
        assert_eq!(slice.last(), Some(&18));
        assert_eq!(slice.get(5), Some(&10));
        assert_eq!(slice.iter().sum::<i32>(), 90); // 0+2+4+6+8+10+12+14+16+18
    }

    #[test]
    fn test_pool_from_fn() {
        let mut arena: Arena<1024> = Arena::new();

        // Create pool with closure initialization
        let pool = arena.alloc_pool_from_fn(5, |i| i as u32 * 10).unwrap();
        
        let slice = arena.get_pool(&pool);
        assert_eq!(slice, &[0, 10, 20, 30, 40]);
    }
}

/// Tests for ID functionality
mod id_tests {
    use super::*;

    #[test]
    fn test_arena_id_properties() {
        let mut arena: Arena<1024> = Arena::new();

        let id = arena.alloc(42u32).unwrap();
        assert_eq!(id.size(), 4);
        assert!(id.is_valid());
        assert_eq!(id.info(), (id.offset(), 4));
    }

    #[test]
    fn test_arena_id_equality() {
        let mut arena: Arena<1024> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let id2 = arena.alloc(42u32).unwrap();

        // Same value but different locations
        assert_ne!(id1, id2);
        assert_eq!(id1, id1);
    }

    #[test]
    fn test_arena_id_clone() {
        let mut arena: Arena<1024> = Arena::new();

        let id1 = arena.alloc(42u32).unwrap();
        let id2 = id1; // Copy

        assert_eq!(id1, id2);
        assert_eq!(*arena.get(&id1), *arena.get(&id2));
    }
}

/// Integration tests combining multiple features
mod integration_tests {
    use super::*;

    #[test]
    fn test_mixed_allocations() {
        let mut arena: Arena<1024> = Arena::new();

        // Mix single allocations and pools
        let single1 = arena.alloc(42u32).unwrap();
        let pool1 = arena.alloc_pool::<u8>(10).unwrap();
        let single2 = arena.alloc(3.14f64).unwrap();
        let pool2 = arena.alloc_pool::<u16>(5).unwrap();

        // Initialize data
        *arena.get_mut(&single1) = 100;
        
        let slice1 = arena.get_pool_mut(&pool1);
        for (i, val) in slice1.iter_mut().enumerate() {
            *val = i as u8;
        }

        *arena.get_mut(&single2) = 2.718;

        let slice2 = arena.get_pool_mut(&pool2);
        for (i, val) in slice2.iter_mut().enumerate() {
            *val = (i * 1000) as u16;
        }

        // Verify all data is correct
        assert_eq!(*arena.get(&single1), 100);
        assert_eq!(*arena.get(&single2), 2.718);
        
        let slice1 = arena.get_pool(&pool1);
        assert_eq!(slice1, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        
        let slice2 = arena.get_pool(&pool2);
        assert_eq!(slice2, &[0, 1000, 2000, 3000, 4000]);
    }

    #[test]
    fn test_arena_lifecycle() {
        let mut arena: Arena<1024> = Arena::new();

        // Phase 1: Initial allocations
        let id1 = arena.alloc(42u32).unwrap();
        let pool1 = arena.alloc_pool::<u8>(10).unwrap();
        
        assert_eq!(arena.allocation_count(), 2);
        assert_eq!(*arena.get(&id1), 42);
        assert_eq!(pool1.len(), 10);

        // Phase 2: Clear and reallocate
        arena.clear();
        assert_eq!(arena.allocation_count(), 0);
        assert_eq!(arena.used(), 0);

        // Phase 3: New allocations should work
        let id2 = arena.alloc(100u32).unwrap();
        let pool2 = arena.alloc_pool::<u16>(5).unwrap();
        
        assert_eq!(arena.allocation_count(), 2);
        assert_eq!(*arena.get(&id2), 100);
        assert_eq!(pool2.len(), 5);
    }

    #[test]
    fn test_memory_layout() {
        let mut arena: Arena<1024> = Arena::new();

        // Test that allocations are laid out sequentially
        let id1 = arena.alloc(1u8).unwrap();
        let id2 = arena.alloc(2u8).unwrap();
        let id3 = arena.alloc(3u8).unwrap();

        assert_eq!(id1.offset(), 0);
        assert_eq!(id2.offset(), 1);
        assert_eq!(id3.offset(), 2);

        assert_eq!(*arena.get(&id1), 1);
        assert_eq!(*arena.get(&id2), 2);
        assert_eq!(*arena.get(&id3), 3);
    }

    #[test]
    fn test_large_allocation() {
        let mut arena: Arena<1024> = Arena::new();

        // Allocate a large structure
        let large_array = [42u32; 100]; // 400 bytes
        let id = arena.alloc(large_array).unwrap();

        let retrieved = arena.get(&id);
        assert_eq!(retrieved.len(), 100);
        assert_eq!(retrieved[0], 42);
        assert_eq!(retrieved[99], 42);
    }

    #[test]
    fn test_stress_small_allocations() {
        let mut arena: Arena<1024> = Arena::new();

        let mut ids = [None; 1024];
        let mut count = 0;
        
        // Allocate as many u8s as possible
        for i in 0..255 {
            if let Some(id) = arena.alloc(i as u8) {
                ids[count] = Some(id);
                count += 1;
            } else {
                break;
            }
        }

        // Verify all allocations
        for i in 0..count {
            if let Some(id) = ids[i] {
                assert_eq!(*arena.get(&id), i as u8);
            }
        }

        // Should have allocated quite a few
        assert!(count > 200);
    }
}

/// Edge case and error condition tests
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_zero_sized_types() {
        let mut arena: Arena<1024> = Arena::new();

        // Unit type has zero size
        let id = arena.alloc(()).unwrap();
        assert_eq!(id.size(), 0);
        assert_eq!(*arena.get(&id), ());
    }

    #[test]
    fn test_tiny_arena() {
        let mut arena: Arena<1> = Arena::new();

        // Should be able to allocate a single byte
        let id = arena.alloc(42u8).unwrap();
        assert_eq!(*arena.get(&id), 42);

        // Should fail to allocate anything else
        let result = arena.alloc(99u8);
        assert!(result.is_none());
    }

    #[test]
    fn test_alignment_edge_cases() {
        let mut arena: Arena<1024> = Arena::new();

        // Test various alignment requirements
        let _u8_id = arena.alloc(1u8).unwrap();      // 1-byte aligned
        let _u16_id = arena.alloc(2u16).unwrap();    // 2-byte aligned
        let _u32_id = arena.alloc(3u32).unwrap();    // 4-byte aligned
        let _u64_id = arena.alloc(4u64).unwrap();    // 8-byte aligned

        // All should work without issues
        assert!(arena.used() > 0);
    }

    #[test]
    fn test_empty_arena_stats() {
        let arena: Arena<1024> = Arena::new();

        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 1024);
        assert_eq!(arena.allocation_count(), 0);
    }
}