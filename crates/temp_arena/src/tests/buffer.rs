use super::super::*;

#[test]
fn test_buffer_with_capacity() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let buffer = TempBuffer::<i32>::with_capacity(&mut arena, 5).unwrap();
    
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.capacity(), 5);
    assert!(buffer.is_empty());
    assert!(!buffer.is_full());
    assert_eq!(buffer.remaining(), 5);
}

#[test]
fn test_buffer_push_and_pop() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    
    // Push items
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer.remaining(), 1);
    assert!(!buffer.is_empty());
    assert!(!buffer.is_full());
    
    // Check contents
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice, &[10, 20]);
    
    // Pop items
    assert_eq!(buffer.pop(&arena), Some(20));
    assert_eq!(buffer.pop(&arena), Some(10));
    assert_eq!(buffer.pop(&arena), None);
    
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
}

#[test]
fn test_buffer_full_capacity() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 2).unwrap();
    
    // Fill to capacity
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    
    assert!(buffer.is_full());
    assert_eq!(buffer.remaining(), 0);
    
    // Try to push when full
    let result = buffer.push(&mut arena, 3);
    match result {
        Err(TempArenaError::BufferFull) => {
            // Expected
        }
        _ => panic!("Expected BufferFull error"),
    }
}

#[test]
fn test_buffer_from_iter() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let data = [1, 2, 3, 4, 5];
    let buffer = TempBuffer::from_iter(&mut arena, data.iter().copied()).unwrap();
    
    assert_eq!(buffer.len(), 5);
    assert_eq!(buffer.capacity(), 5);
    assert!(buffer.is_full());
    
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice, &[1, 2, 3, 4, 5]);
}

#[test]
fn test_buffer_from_fn() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let buffer = TempBuffer::from_fn(&mut arena, 4, |i| (i * i) as i32).unwrap();
    
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.capacity(), 4);
    
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice, &[0, 1, 4, 9]);
}

#[test]
fn test_buffer_empty() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let buffer = TempBuffer::from_iter(&mut arena, core::iter::empty::<i32>()).unwrap();
    
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.capacity(), 0);
    assert!(buffer.is_empty());
    assert!(buffer.is_full()); // Zero capacity means always full
    
    let slice = buffer.as_slice(&arena).unwrap();
    assert!(slice.is_empty());
}

#[test]
fn test_buffer_get_element() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 5).unwrap();
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    // Test valid indices
    assert_eq!(*buffer.get(&arena, 0).unwrap(), 10);
    assert_eq!(*buffer.get(&arena, 1).unwrap(), 20);
    assert_eq!(*buffer.get(&arena, 2).unwrap(), 30);
    
    // Test out of bounds (beyond length)
    assert!(buffer.get(&arena, 3).is_none());
    assert!(buffer.get(&arena, 4).is_none());
    assert!(buffer.get(&arena, 100).is_none());
}

#[test]
fn test_buffer_mutable_access() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    
    // Modify through mutable slice
    {
        let slice_mut = buffer.as_slice_mut(&mut arena).unwrap();
        slice_mut[1] = 99;
    }
    
    // Check modification
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice, &[1, 99, 3]);
}

#[test]
fn test_buffer_get_mut_element() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    buffer.push(&mut arena, 5).unwrap();
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 15).unwrap();
    
    // Modify specific element
    *buffer.get_mut(&mut arena, 1).unwrap() = 42;
    
    // Check modification
    assert_eq!(*buffer.get(&arena, 0).unwrap(), 5);
    assert_eq!(*buffer.get(&arena, 1).unwrap(), 42);
    assert_eq!(*buffer.get(&arena, 2).unwrap(), 15);
    
    // Test out of bounds for mutable access
    assert!(buffer.get_mut(&mut arena, 3).is_none());
}

#[test]
fn test_buffer_clear() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 5).unwrap();
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    
    assert_eq!(buffer.len(), 3);
    assert!(!buffer.is_empty());
    
    buffer.clear();
    
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
    assert_eq!(buffer.capacity(), 5); // Capacity unchanged
    
    // Can push again after clear
    buffer.push(&mut arena, 99).unwrap();
    assert_eq!(buffer.len(), 1);
    assert_eq!(*buffer.get(&arena, 0).unwrap(), 99);
}

#[test]
fn test_buffer_memory_reuse() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    
    // Push and pop
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    assert_eq!(buffer.pop(&arena), Some(30));
    assert_eq!(buffer.pop(&arena), Some(20));
    assert_eq!(buffer.len(), 1);
    
    // Push again - should reuse memory
    buffer.push(&mut arena, 40).unwrap();
    buffer.push(&mut arena, 50).unwrap();
    
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice, &[10, 40, 50]);
}

#[test]
fn test_buffer_different_types() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Test with floats
    let mut float_buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    float_buffer.push(&mut arena, 3.14f32).unwrap();
    float_buffer.push(&mut arena, 2.71f32).unwrap();
    
    let float_slice = float_buffer.as_slice(&arena).unwrap();
    assert_eq!(float_slice, &[3.14, 2.71]);
    
    // Test with strings
    let str_data = ["hello", "world", "test"];
    let str_buffer = TempBuffer::from_iter(&mut arena, str_data.iter().copied()).unwrap();
    let str_slice = str_buffer.as_slice(&arena).unwrap();
    assert_eq!(str_slice, &["hello", "world", "test"]);
}

#[test]
fn test_buffer_with_structs() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    struct Point {
        x: i32,
        y: i32,
    }
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    buffer.push(&mut arena, Point { x: 0, y: 0 }).unwrap();
    buffer.push(&mut arena, Point { x: 1, y: 2 }).unwrap();
    buffer.push(&mut arena, Point { x: 3, y: 4 }).unwrap();
    
    assert_eq!(buffer.len(), 3);
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice[0], Point { x: 0, y: 0 });
    assert_eq!(slice[1], Point { x: 1, y: 2 });
    assert_eq!(slice[2], Point { x: 3, y: 4 });
    
    // Test pop
    assert_eq!(buffer.pop(&arena), Some(Point { x: 3, y: 4 }));
}

#[test]
fn test_buffer_with_different_index_types() {
    // Test with u8 index
    let mut arena_u8: TempArena<256, u8> = TempArena::new();
    let mut buffer_u8 = TempBuffer::with_capacity(&mut arena_u8, 3).unwrap();
    buffer_u8.push(&mut arena_u8, 1).unwrap();
    buffer_u8.push(&mut arena_u8, 2).unwrap();
    assert_eq!(buffer_u8.as_slice(&arena_u8).unwrap(), &[1, 2]);
    
    // Test with u16 index
    let mut arena_u16: TempArena<1024, u16> = TempArena::new();
    let mut buffer_u16 = TempBuffer::with_capacity(&mut arena_u16, 3).unwrap();
    buffer_u16.push(&mut arena_u16, 4).unwrap();
    buffer_u16.push(&mut arena_u16, 5).unwrap();
    assert_eq!(buffer_u16.as_slice(&arena_u16).unwrap(), &[4, 5]);
}

#[test]
fn test_buffer_large_data() {
    let mut arena: TempArena<4096, u32> = TempArena::new();
    
    // Create a larger buffer
    let mut buffer = TempBuffer::with_capacity(&mut arena, 100).unwrap();
    
    // Push many items
    for i in 0..100 {
        buffer.push(&mut arena, i as i32).unwrap();
    }
    
    assert_eq!(buffer.len(), 100);
    assert!(buffer.is_full());
    
    let slice = buffer.as_slice(&arena).unwrap();
    assert_eq!(slice.len(), 100);
    
    // Check some values
    assert_eq!(slice[0], 0);
    assert_eq!(slice[50], 50);
    assert_eq!(slice[99], 99);
    
    // Pop some items
    for expected in (90..100).rev() {
        assert_eq!(buffer.pop(&arena), Some(expected));
    }
    
    assert_eq!(buffer.len(), 90);
}

#[test]
fn test_buffer_out_of_space() {
    let mut arena: TempArena<32, u32> = TempArena::new(); // Very small arena
    
    // Try to allocate more than the arena can hold
    // Each i64 is 8 bytes, so 5 of them = 40 bytes > 32 bytes available
    let result = TempBuffer::<i64>::with_capacity(&mut arena, 5);
    
    assert!(result.is_err());
    match result {
        Err(TempArenaError::OutOfSpace { .. }) => {
            // Expected error
        }
        _ => panic!("Expected OutOfSpace error"),
    }
}

#[test]
fn test_buffer_multiple_buffers() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Create multiple buffers in the same arena
    let mut buffer1 = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    let mut buffer2 = TempBuffer::with_capacity(&mut arena, 2).unwrap();
    let mut buffer3 = TempBuffer::with_capacity(&mut arena, 4).unwrap();
    
    // Fill them with different data
    buffer1.push(&mut arena, 1).unwrap();
    buffer1.push(&mut arena, 2).unwrap();
    
    buffer2.push(&mut arena, 10.0).unwrap();
    buffer2.push(&mut arena, 20.0).unwrap();
    
    buffer3.push(&mut arena, "a").unwrap();
    buffer3.push(&mut arena, "b").unwrap();
    buffer3.push(&mut arena, "c").unwrap();
    
    // All buffers should be accessible
    assert_eq!(buffer1.as_slice(&arena).unwrap(), &[1, 2]);
    assert_eq!(buffer2.as_slice(&arena).unwrap(), &[10.0, 20.0]);
    assert_eq!(buffer3.as_slice(&arena).unwrap(), &["a", "b", "c"]);
    
    // Test individual access
    assert_eq!(*buffer1.get(&arena, 1).unwrap(), 2);
    assert_eq!(*buffer2.get(&arena, 0).unwrap(), 10.0);
    assert_eq!(*buffer3.get(&arena, 2).unwrap(), "c");
}

#[test]
fn test_buffer_clone_copy_semantics() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer1 = TempBuffer::with_capacity(&mut arena, 3).unwrap();
    buffer1.push(&mut arena, 1).unwrap();
    buffer1.push(&mut arena, 2).unwrap();
    
    // TempBuffer should be Copy, so we can copy it
    let buffer2 = buffer1;
    
    // Both should work and reference the same data
    assert_eq!(buffer1.as_slice(&arena).unwrap(), &[1, 2]);
    assert_eq!(buffer2.as_slice(&arena).unwrap(), &[1, 2]);
    assert_eq!(buffer1.len(), buffer2.len());
    assert_eq!(buffer1.capacity(), buffer2.capacity());
}

#[test]
fn test_buffer_push_pop_cycle() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let mut buffer = TempBuffer::with_capacity(&mut arena, 2).unwrap();
    
    // Fill, empty, fill again
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    
    assert_eq!(buffer.pop(&arena), Some(2));
    assert_eq!(buffer.pop(&arena), Some(1));
    assert!(buffer.is_empty());
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    
    assert_eq!(buffer.as_slice(&arena).unwrap(), &[10, 20]);
    assert!(buffer.is_full());
}

#[test]
fn test_buffer_comprehensive_demo() {
    let mut arena: TempArena<1024, u32> = TempArena::new();

    // Create a buffer for managing temporary game entities
    let mut entities = TempBuffer::<i32>::with_capacity(&mut arena, 10).unwrap();
    
    assert_eq!(entities.capacity(), 10);
    assert_eq!(entities.len(), 0);
    assert!(entities.is_empty());
    assert_eq!(entities.remaining(), 10);

    // Add some entities during frame processing
    entities.push(&mut arena, 100).unwrap(); // Player
    entities.push(&mut arena, 200).unwrap(); // Enemy 1
    entities.push(&mut arena, 201).unwrap(); // Enemy 2
    entities.push(&mut arena, 300).unwrap(); // Powerup
    
    assert_eq!(entities.len(), 4);
    assert_eq!(entities.remaining(), 6);
    assert!(!entities.is_full());

    // Process entities (read access)
    let entity_slice = entities.as_slice(&arena).unwrap();
    assert_eq!(entity_slice, &[100, 200, 201, 300]);
    
    // Access individual entities
    assert_eq!(*entities.get(&arena, 0).unwrap(), 100); // Player
    assert_eq!(*entities.get(&arena, 3).unwrap(), 300); // Powerup
    
    // Remove powerup (collected!)
    let collected = entities.pop(&arena).unwrap();
    assert_eq!(collected, 300);
    assert_eq!(entities.len(), 3);

    // Modify entity (damage enemy)
    let entities_mut = entities.as_slice_mut(&mut arena).unwrap();
    entities_mut[1] = 150; // Enemy took damage
    
    assert_eq!(entities.as_slice(&arena).unwrap(), &[100, 150, 201]);

    // Add more entities (they reuse the memory from the popped powerup)
    entities.push(&mut arena, 400).unwrap(); // New pickup
    entities.push(&mut arena, 500).unwrap(); // Another enemy
    
    assert_eq!(entities.as_slice(&arena).unwrap(), &[100, 150, 201, 400, 500]);
    assert_eq!(entities.len(), 5);

    // End of frame - clear all entities for next frame
    entities.clear();
    assert_eq!(entities.len(), 0);
    assert!(entities.is_empty());
    assert_eq!(entities.capacity(), 10); // Capacity unchanged
    assert_eq!(entities.remaining(), 10);

    // Next frame - entities can be reused
    entities.push(&mut arena, 1000).unwrap();
    entities.push(&mut arena, 2000).unwrap();
    
    // Old data is still in memory but not accessible
    // Only the new data [1000, 2000] is visible
    assert_eq!(entities.as_slice(&arena).unwrap(), &[1000, 2000]);
    assert_eq!(entities.len(), 2);
    
    // This demonstrates the efficient memory reuse pattern:
    // - No allocations after initial capacity setup
    // - Pop just decrements counter (no data movement)
    // - Push overwrites old data in place
    // - Perfect for frame-based temporary data!
}