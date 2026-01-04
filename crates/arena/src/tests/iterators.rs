use super::*;
use crate::Buffer;

#[test]
fn test_arena_iter_slice_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let empty_slice = arena.alloc_slice::<u32>(&[]).unwrap();
    let iter = arena.iter_slice(empty_slice).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_arena_iter_slice_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32 * 10).unwrap();

    let mut iter = arena.iter_slice(slice).unwrap();
    assert_eq!(iter.next(), Some(&0));
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next(), Some(&20));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), Some(&40));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_arena_iter_slice_invalid_after_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(3, |i| i as u32).unwrap();

    arena.clear();
    assert!(arena.iter_slice(slice).is_err());
}

#[test]
fn test_arena_iter_slice_range_full() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    let iter = arena.iter_slice_range(slice, 0, 5).unwrap();
    assert_eq!(iter.count(), 5);
}

#[test]
fn test_arena_iter_slice_range_partial() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(10, |i| i as u32).unwrap();

    let mut iter = arena.iter_slice_range(slice, 2, 5).unwrap();
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&4));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_arena_iter_slice_range_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    let iter = arena.iter_slice_range(slice, 3, 3).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_arena_iter_slice_range_out_of_bounds() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(10, |i| i as u32).unwrap();

    // End beyond slice length
    assert!(arena.iter_slice_range(slice, 5, 15).is_err());

    // Start > end
    assert!(arena.iter_slice_range(slice, 8, 5).is_err());
}

#[test]
fn test_arena_iter_slice_range_invalid_after_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    arena.clear();
    assert!(arena.iter_slice_range(slice, 0, 3).is_err());
}

#[test]
fn test_buffer_iter_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let buffer = Buffer::<i32>::new(&mut arena, 5).unwrap();

    let iter = buffer.items(&mut arena).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_buffer_iter_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    let mut iter = buffer.items(&mut arena).unwrap();
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next(), Some(&20));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), None);

    // Verify it only iterates over used elements, not full capacity
    assert_eq!(buffer.items(&mut arena).unwrap().count(), 3);
}

#[test]
fn test_buffer_iter_after_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 42).unwrap();
    buffer.clear();

    let iter = buffer.items(&mut arena).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_iter_enumerate_pattern() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(4, |i| (i + 1) * 100).unwrap();

    for (i, &value) in arena.iter_slice(slice).unwrap().enumerate() {
        assert_eq!(value, (i + 1) * 100);
    }
}

#[test]
fn test_buffer_drain_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::<i32>::new(&mut arena, 5).unwrap();

    let mut drain = buffer.drain(&mut arena);
    assert_eq!(drain.len(), 0);
    assert_eq!(drain.next(), None);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_buffer_drain_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    let mut drain = buffer.drain(&mut arena);
    assert_eq!(drain.len(), 3);
    assert_eq!(drain.next(), Some(10));
    assert_eq!(drain.len(), 2);
    assert_eq!(drain.next(), Some(20));
    assert_eq!(drain.next(), Some(30));
    assert_eq!(drain.next(), None);

    // Buffer should be empty after drain
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_buffer_drain_size_hints() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();

    let drain = buffer.drain(&mut arena);
    let (lower, upper) = drain.size_hint();
    assert_eq!(lower, 2);
    assert_eq!(upper, Some(2));
}

#[test]
fn test_buffer_drain_collect_pattern() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 4).unwrap();

    buffer.push(&mut arena, 100).unwrap();
    buffer.push(&mut arena, 200).unwrap();
    buffer.push(&mut arena, 300).unwrap();

    let drain = buffer.drain(&mut arena);
    let mut sum = 0;
    for value in drain {
        sum += value;
    }
    assert_eq!(sum, 600);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_buffer_pop_basic() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    // Pop from empty buffer should return None
    assert_eq!(buffer.pop(&mut arena), None);
    assert_eq!(buffer.len(), 0);

    // Add some elements
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    assert_eq!(buffer.len(), 3);

    // Pop elements in reverse order
    assert_eq!(buffer.pop(&mut arena), Some(30));
    assert_eq!(buffer.len(), 2);

    assert_eq!(buffer.pop(&mut arena), Some(20));
    assert_eq!(buffer.len(), 1);

    assert_eq!(buffer.pop(&mut arena), Some(10));
    assert_eq!(buffer.len(), 0);

    // Pop from empty buffer again
    assert_eq!(buffer.pop(&mut arena), None);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_buffer_pop_after_operations() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 10).unwrap();

    // Fill buffer
    for i in 0..5 {
        buffer.push(&mut arena, i * 10).unwrap();
    }
    assert_eq!(buffer.len(), 5);

    // Pop some elements
    assert_eq!(buffer.pop(&mut arena), Some(40));
    assert_eq!(buffer.pop(&mut arena), Some(30));
    assert_eq!(buffer.len(), 3);

    // Push more elements
    buffer.push(&mut arena, 100).unwrap();
    buffer.push(&mut arena, 200).unwrap();
    assert_eq!(buffer.len(), 5);

    // Pop should get the most recently added
    assert_eq!(buffer.pop(&mut arena), Some(200));
    assert_eq!(buffer.pop(&mut arena), Some(100));
    assert_eq!(buffer.pop(&mut arena), Some(20));
    assert_eq!(buffer.len(), 2);
}

// Additional tests to showcase ArenaOps flexibility
#[test]
fn test_buffer_with_arena_ref() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::<i32>::new(&mut arena, 5).unwrap();
    
    // Push using Arena
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    
    // Create an ArenaRef and use it for operations
    let mut arena_ref = arena.as_ref();
    buffer.push(&mut arena_ref, 30).unwrap();
    assert_eq!(buffer.pop(&mut arena_ref), Some(30));
    
    // Verify we can still use the original arena
    assert_eq!(buffer.pop(&mut arena), Some(20));
    assert_eq!(buffer.len(), 1);
}

#[test] 
fn test_buffer_items_with_arena_ref() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();
    
    buffer.push(&mut arena, 100).unwrap();
    buffer.push(&mut arena, 200).unwrap();
    
    // Use ArenaRef for iteration
    let mut arena_ref = arena.as_ref();
    let mut iter = buffer.items(&mut arena_ref).unwrap();
    assert_eq!(iter.next(), Some(&100));
    assert_eq!(iter.next(), Some(&200));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_buffer_drain_with_arena_ref() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    // Use ArenaRef for draining
    let mut arena_ref = arena.as_ref();
    let mut drain = buffer.drain(&mut arena_ref);
    assert_eq!(drain.next(), Some(10));
    assert_eq!(drain.next(), Some(20));
    assert_eq!(drain.next(), Some(30));
    assert_eq!(drain.next(), None);
    
    assert_eq!(buffer.len(), 0);
}

// Test that demonstrates using a generic function with ArenaOps
fn generic_buffer_operations<A, I, M>(arena: &mut A, values: &[i32]) -> Buffer<i32, I, M>
where
    A: ArenaOps<I, M>,
    I: ArenaIndex,
{
    let mut buffer = Buffer::new(arena, I::try_from(values.len() * 2).ok().unwrap()).unwrap();
    for &val in values {
        buffer.push(arena, val).unwrap();
    }
    buffer
}

#[test]
fn test_generic_arena_ops_function() {
    let mut arena: Arena<1024> = Arena::new();
    
    // Test with Arena directly
    let values = [10, 20, 30, 40];
    let mut buffer = generic_buffer_operations(&mut arena, &values);
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.pop(&mut arena), Some(40));
    
    // Test with ArenaRef
    let mut arena_ref = arena.as_ref();
    let more_values = [50, 60];
    for &val in &more_values {
        buffer.push(&mut arena_ref, val).unwrap();
    }
    assert_eq!(buffer.len(), 5);
}

#[test]
fn test_buffer_as_slice_with_arena_ops() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    // Get slice using Arena
    let slice = buffer.as_slice(&mut arena).unwrap();
    assert_eq!(slice, &[10, 20, 30]);
    
    // Get mutable slice using ArenaRef
    let mut arena_ref = arena.as_ref();
    let slice_mut = buffer.as_slice_mut(&mut arena_ref).unwrap();
    slice_mut[1] = 25;
    
    // Verify change persisted
    let slice = buffer.as_slice(&mut arena).unwrap();
    assert_eq!(slice, &[10, 25, 30]);
}

#[test]
fn test_buffer_resize_with_arena_ops() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer: Buffer<i32> = Buffer::new(&mut arena, 10).unwrap();
    
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    assert_eq!(buffer.len(), 3);
    
    // Resize using ArenaRef
    let mut arena_ref = arena.as_ref();
    buffer.resize(&mut arena_ref, 5);
    assert_eq!(buffer.len(), 5);
    
    // Check that all elements are set to default (0 for i32) after resize
    // Note: Current implementation resets all elements when resizing up
    let slice = buffer.as_slice(&mut arena).unwrap();
    assert_eq!(slice[0], 0);
    assert_eq!(slice[1], 0);
    assert_eq!(slice[2], 0);
    assert_eq!(slice[3], 0);
    assert_eq!(slice[4], 0);
}

#[test]
fn test_mixed_arena_ops_operations() {
    let mut arena: Arena<2048> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 8).unwrap();
    
    // Push with Arena
    buffer.push(&mut arena, 100).unwrap();
    
    // Create ArenaRef scope for some operations
    {
        let mut arena_ref = arena.as_ref();
        buffer.push(&mut arena_ref, 200).unwrap();
        
        // Use ArenaRef for iteration
        let mut iter = buffer.items(&mut arena_ref).unwrap();
        assert_eq!(iter.next(), Some(&100));
        assert_eq!(iter.next(), Some(&200));
    }
    
    // Back to Arena for more operations
    buffer.push(&mut arena, 300).unwrap();
    
    // Pop with Arena
    assert_eq!(buffer.pop(&mut arena), Some(300));
    
    // Drain with ArenaRef in a new scope
    let mut arena_ref = arena.as_ref();
    let mut drain = buffer.drain(&mut arena_ref);
    assert_eq!(drain.next(), Some(100));
    assert_eq!(drain.next(), Some(200));
    assert_eq!(drain.next(), None);
    
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_buffer_get_method() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 10).unwrap();
    
    // Test get on empty buffer
    assert_eq!(buffer.get(&arena, 0), None);
    assert_eq!(buffer.get(&arena, 5), None);
    
    // Add some elements
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    buffer.push(&mut arena, 40).unwrap();
    buffer.push(&mut arena, 50).unwrap();
    
    // Test getting valid indices
    assert_eq!(buffer.get(&arena, 0), Some(10));
    assert_eq!(buffer.get(&arena, 1), Some(20));
    assert_eq!(buffer.get(&arena, 2), Some(30));
    assert_eq!(buffer.get(&arena, 3), Some(40));
    assert_eq!(buffer.get(&arena, 4), Some(50));
    
    // Test getting out of bounds
    assert_eq!(buffer.get(&arena, 5), None);
    assert_eq!(buffer.get(&arena, 100), None);
    
    // Test with ArenaRef
    let arena_ref = arena.as_ref();
    assert_eq!(buffer.get(&arena_ref, 0), Some(10));
    assert_eq!(buffer.get(&arena_ref, 4), Some(50));
    assert_eq!(buffer.get(&arena_ref, 5), None);
    
    // Test after pop
    buffer.pop(&mut arena).unwrap();
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.get(&arena, 3), Some(40));
    assert_eq!(buffer.get(&arena, 4), None); // This index is now out of bounds
    
    // Test after clear
    buffer.clear();
    assert_eq!(buffer.get(&arena, 0), None);
}