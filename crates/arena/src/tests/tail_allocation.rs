use super::*;

#[test]
fn test_tail_alloc_basic() {
    let mut arena: Arena<1024> = Arena::new();

    // Create some data in a slice
    let source_slice = arena.alloc_slice_from_fn(10, |i| (i + 65) as u8).unwrap(); // "ABCDEFGHIJ"

    // Use tail allocation to copy it
    let copied_slice = arena.copy_slice_via_tail(&source_slice, 10u32).unwrap();

    // Verify the copy is correct
    let source_data = arena.get_slice(&source_slice).unwrap();
    let copied_data = arena.get_slice(&copied_slice).unwrap();

    assert_eq!(source_data, copied_data);
    assert_eq!(copied_data.len(), 10);
    assert_eq!(copied_data[0], 65); // 'A'
    assert_eq!(copied_data[9], 74); // 'J'
}

#[test]
fn test_tail_alloc_partial_copy() {
    let mut arena: Arena<1024> = Arena::new();

    // Create a longer slice but only copy part of it
    let source_slice = arena.alloc_slice_from_fn(20, |i| (i + 48) as u8).unwrap(); // "0123456789..."

    // Only copy first 5 bytes
    let copied_slice = arena.copy_slice_via_tail(&source_slice, 5u32).unwrap();

    let copied_data = arena.get_slice(&copied_slice).unwrap();

    assert_eq!(copied_data.len(), 5);
    assert_eq!(copied_data[0], 48); // '0'
    assert_eq!(copied_data[4], 52); // '4'
}

#[test]
fn test_tail_alloc_with_existing_allocations() {
    let mut arena: Arena<512> = Arena::new();

    // Make some existing allocations
    let _existing1 = arena.alloc_slice_from_fn(100, |i| i as u8).unwrap();
    let _existing2 = arena.alloc_slice_from_fn(50, |i| (i + 100) as u8).unwrap();

    // Create source data
    let source_slice = arena.alloc_slice_from_fn(30, |i| (i + 200) as u8).unwrap();

    // Use tail allocation - should work with existing allocations
    let copied_slice = arena.copy_slice_via_tail(&source_slice, 30u32).unwrap();

    let source_data = arena.get_slice(&source_slice).unwrap();
    let copied_data = arena.get_slice(&copied_slice).unwrap();

    assert_eq!(source_data, copied_data);
    assert_eq!(copied_data.len(), 30);
}

#[test]
fn test_tail_alloc_insufficient_space() {
    let mut arena: Arena<64> = Arena::new(); // Very small arena

    // Fill up most of the arena
    let _filler = arena.alloc_slice_from_fn(50, |i| i as u8).unwrap();

    // Create a small source slice
    let source_slice = arena.alloc_slice_from_fn(8, |i| (i + 100) as u8).unwrap();

    // Try to copy - should fail because we need space for both tail copy AND final allocation
    // With only ~6 bytes left, we can't fit 8 bytes for tail + 8 bytes for final
    let result = arena.copy_slice_via_tail(&source_slice, 8u32);

    assert!(result.is_err());
    match result {
        Err(ArenaErr::OutOfSpace { requested: _, available: _ }) => {
            // Expected
        }
        _ => panic!("Expected OutOfSpace error"),
    }
}

#[test]
fn test_tail_alloc_exactly_fits() {
    let mut arena: Arena<128> = Arena::new();

    // Calculate space carefully - need room for source + tail copy + final copy
    let source_size = 20u32;
    let _source = arena.alloc_slice_from_fn(source_size as usize, |i| (i + 65) as u8).unwrap();

    // Fill arena to leave exactly enough space for tail allocation
    let _used = arena.used();
    let remaining = arena.remaining();

    // We need: tail_space + final_space >= 2 * source_size
    // Leave exactly enough space
    let fill_amount = remaining - (2 * source_size.to_usize());
    if fill_amount > 0 {
        let fill_size = u32::try_from(fill_amount).unwrap();
        let _filler = arena.alloc_slice_from_fn(fill_size as usize, |i| i as u8).unwrap();
    }

    // Now try the tail allocation - should just barely fit
    let copied_slice = arena.copy_slice_via_tail(&_source, source_size).unwrap();

    let copied_data = arena.get_slice(&copied_slice).unwrap();
    assert_eq!(copied_data.len(), 20);
}

#[test]
fn test_tail_alloc_zero_length() {
    let mut arena: Arena<1024> = Arena::new();

    // Create a slice but copy zero length
    let source_slice = arena.alloc_slice_from_fn(10, |i| (i + 100) as u8).unwrap();

    let copied_slice = arena.copy_slice_via_tail(&source_slice, 0u32).unwrap();

    let copied_data = arena.get_slice(&copied_slice).unwrap();
    assert_eq!(copied_data.len(), 0);
}

#[test]
fn test_tail_alloc_clears_tail_on_success() {
    let mut arena: Arena<1024> = Arena::new();

    // Create and copy some data
    let source_slice = arena.alloc_slice_from_fn(20, |i| i as u8).unwrap();
    let _copied = arena.copy_slice_via_tail(&source_slice, 20u32).unwrap();

    // Tail should be cleared - verify by checking we can use full remaining space
    let remaining = arena.remaining();

    // Should be able to allocate all remaining space (tail is reset)
    let big_alloc_size = u32::try_from(remaining - 4).unwrap(); // Leave a bit of room
    let _big_alloc = arena.alloc_slice_from_fn(big_alloc_size as usize, |i| i as u8);

    // Should succeed if tail was properly cleared
    assert!(_big_alloc.is_ok());
}

#[test]
fn test_tail_alloc_different_types() {
    let mut arena: Arena<1024> = Arena::new();

    // Test with u32 values
    let source_u32 = arena.alloc_slice_from_fn(5, |i| (i * 100) as u32).unwrap();
    let copied_u32 = arena.copy_slice_via_tail(&source_u32, 5u32).unwrap();

    let source_data_u32 = arena.get_slice(&source_u32).unwrap();
    let copied_data_u32 = arena.get_slice(&copied_u32).unwrap();

    assert_eq!(source_data_u32, copied_data_u32);
    assert_eq!(copied_data_u32, &[0, 100, 200, 300, 400]);

    // Test with u16 values
    let source_u16 = arena.alloc_slice_from_fn(3, |i| (i + 1000) as u16).unwrap();
    let copied_u16 = arena.copy_slice_via_tail(&source_u16, 3u32).unwrap();

    let source_data_u16 = arena.get_slice(&source_u16).unwrap();
    let copied_data_u16 = arena.get_slice(&copied_u16).unwrap();

    assert_eq!(source_data_u16, copied_data_u16);
    assert_eq!(copied_data_u16, &[1000, 1001, 1002]);
}

#[test]
fn test_tail_alloc_arena_clear_resets_tail() {
    let mut arena: Arena<1024> = Arena::new();

    // Do some allocations
    let _data = arena.alloc_slice_from_fn(100, |i| i as u8).unwrap();

    // Clear arena
    arena.clear();

    // After clear, should be able to use full capacity again including tail allocation
    let large_source = arena.alloc_slice_from_fn(400, |i| i as u8).unwrap();
    let copied = arena.copy_slice_via_tail(&large_source, 400u32);

    assert!(copied.is_ok());

    let copied_data = arena.get_slice(&copied.unwrap()).unwrap();
    assert_eq!(copied_data.len(), 400);
}

#[test]
fn test_tail_alloc_stress_multiple_operations() {
    let mut arena: Arena<2048> = Arena::new();

    // Perform multiple tail allocation operations
    for i in 0..10 {
        let size = (i + 1) * 10;
        let source = arena.alloc_slice_from_fn(size, |j| (j + i * 50) as u8).unwrap();
        let copied = arena.copy_slice_via_tail(&source, size as u32).unwrap();

        let source_data = arena.get_slice(&source).unwrap();
        let copied_data = arena.get_slice(&copied).unwrap();

        assert_eq!(source_data, copied_data);
        assert_eq!(copied_data.len(), size);
    }
}
