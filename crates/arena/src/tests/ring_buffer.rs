use super::*;
use crate::RingBuffer;

#[test]
fn test_ring_buffer_new() {
    let mut arena: Arena<1024> = Arena::new();
    let buffer = RingBuffer::<i32>::new(&mut arena, 5).unwrap();

    assert!(buffer.is_empty());
    assert!(!buffer.is_full());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.capacity().to_usize(), 5);
    assert_eq!(buffer.remaining().to_usize(), 5);
    assert_eq!(buffer.used().to_usize(), 0);
}

#[test]
fn test_ring_buffer_push_pop_fifo() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    // Push elements
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    assert!(buffer.is_full());
    assert_eq!(buffer.len(), 3);

    // Pop elements in FIFO order
    assert_eq!(buffer.pop(&arena), Some(10));
    assert_eq!(buffer.pop(&arena), Some(20));
    assert_eq!(buffer.pop(&arena), Some(30));

    assert!(buffer.is_empty());
    assert_eq!(buffer.pop(&arena), None);
}

#[test]
fn test_ring_buffer_push_when_full() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 2).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();

    assert!(buffer.is_full());

    // Should fail when full
    assert!(buffer.push(&mut arena, 30).is_err());
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_ring_buffer_push_overwrite() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    // Fill buffer
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    assert!(buffer.is_full());

    // Overwrite oldest element
    buffer.push_overwrite(&mut arena, 40).unwrap();

    assert!(buffer.is_full());
    assert_eq!(buffer.len(), 3);

    // First element should be 20 (10 was overwritten)
    assert_eq!(buffer.pop(&arena), Some(20));
    assert_eq!(buffer.pop(&arena), Some(30));
    assert_eq!(buffer.pop(&arena), Some(40));
}

#[test]
fn test_ring_buffer_push_overwrite_multiple() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 2).unwrap();

    // Fill buffer
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();

    // Overwrite multiple times
    buffer.push_overwrite(&mut arena, 30).unwrap(); // overwrites 10
    buffer.push_overwrite(&mut arena, 40).unwrap(); // overwrites 20
    buffer.push_overwrite(&mut arena, 50).unwrap(); // overwrites 30

    assert!(buffer.is_full());
    assert_eq!(buffer.pop(&arena), Some(40));
    assert_eq!(buffer.pop(&arena), Some(50));
    assert!(buffer.is_empty());
}

#[test]
fn test_ring_buffer_front_back() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 4).unwrap();

    // Empty buffer
    assert_eq!(buffer.front(&arena), None);
    assert_eq!(buffer.back(&arena), None);

    // Single element
    buffer.push(&mut arena, 10).unwrap();
    assert_eq!(buffer.front(&arena), Some(&10));
    assert_eq!(buffer.back(&arena), Some(&10));

    // Multiple elements
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    assert_eq!(buffer.front(&arena), Some(&10));
    assert_eq!(buffer.back(&arena), Some(&30));

    // After pop
    buffer.pop(&arena);
    assert_eq!(buffer.front(&arena), Some(&20));
    assert_eq!(buffer.back(&arena), Some(&30));
}

#[test]
fn test_ring_buffer_get() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    assert_eq!(buffer.get(&arena, 0u32), Some(&10));
    assert_eq!(buffer.get(&arena, 1u32), Some(&20));
    assert_eq!(buffer.get(&arena, 2u32), Some(&30));
    assert_eq!(buffer.get(&arena, 3u32), None);

    // After pop, indices shift
    buffer.pop(&arena);
    assert_eq!(buffer.get(&arena, 0u32), Some(&20));
    assert_eq!(buffer.get(&arena, 1u32), Some(&30));
    assert_eq!(buffer.get(&arena, 2u32), None);
}

#[test]
fn test_ring_buffer_wrapping() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    // Fill buffer
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();

    // Pop one element
    assert_eq!(buffer.pop(&arena), Some(1));

    // Push another (this should wrap around)
    buffer.push(&mut arena, 4).unwrap();

    // Verify order
    assert_eq!(buffer.get(&arena, 0u32), Some(&2));
    assert_eq!(buffer.get(&arena, 1u32), Some(&3));
    assert_eq!(buffer.get(&arena, 2u32), Some(&4));

    // Pop all and verify order
    assert_eq!(buffer.pop(&arena), Some(2));
    assert_eq!(buffer.pop(&arena), Some(3));
    assert_eq!(buffer.pop(&arena), Some(4));
}

#[test]
fn test_ring_buffer_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();

    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), 2);

    buffer.clear();

    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.remaining().to_usize(), 3);
    assert_eq!(buffer.front(&arena), None);
    assert_eq!(buffer.back(&arena), None);
}

#[test]
fn test_ring_buffer_mixed_operations() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 4).unwrap();

    // Push some elements
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();

    // Pop one
    assert_eq!(buffer.pop(&arena), Some(1));

    // Push more
    buffer.push(&mut arena, 3).unwrap();
    buffer.push(&mut arena, 4).unwrap();
    buffer.push(&mut arena, 5).unwrap();

    assert!(buffer.is_full());

    // Should contain [2, 3, 4, 5]
    assert_eq!(buffer.get(&arena, 0u32), Some(&2));
    assert_eq!(buffer.get(&arena, 1u32), Some(&3));
    assert_eq!(buffer.get(&arena, 2u32), Some(&4));
    assert_eq!(buffer.get(&arena, 3u32), Some(&5));

    // Pop two
    assert_eq!(buffer.pop(&arena), Some(2));
    assert_eq!(buffer.pop(&arena), Some(3));

    // Push with overwrite (buffer has space now)
    buffer.push(&mut arena, 6).unwrap();
    buffer.push(&mut arena, 7).unwrap();

    // Should contain [4, 5, 6, 7]
    assert!(buffer.is_full());
    assert_eq!(buffer.pop(&arena), Some(4));
    assert_eq!(buffer.pop(&arena), Some(5));
    assert_eq!(buffer.pop(&arena), Some(6));
    assert_eq!(buffer.pop(&arena), Some(7));
    assert!(buffer.is_empty());
}

#[test]
fn test_ring_buffer_capacity_one() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 1).unwrap();

    buffer.push(&mut arena, 42).unwrap();
    assert!(buffer.is_full());
    assert_eq!(buffer.front(&arena), Some(&42));
    assert_eq!(buffer.back(&arena), Some(&42));

    // Overwrite
    buffer.push_overwrite(&mut arena, 99).unwrap();
    assert_eq!(buffer.front(&arena), Some(&99));
    assert_eq!(buffer.back(&arena), Some(&99));
    assert_eq!(buffer.pop(&arena), Some(99));
    assert!(buffer.is_empty());
}

#[test]
fn test_ring_buffer_alternating_push_pop() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    // Push first element
    buffer.push(&mut arena, 0).unwrap();

    for i in 1..10 {
        if !buffer.is_full() {
            buffer.push(&mut arena, i).unwrap();
        } else {
            buffer.push_overwrite(&mut arena, i).unwrap();
        }

        if i % 2 == 1 {
            // Pop every other iteration
            buffer.pop(&arena);
        }
    }

    // Should have some elements remaining
    assert!(!buffer.is_empty());
    // Verify we can access elements
    assert!(buffer.get(&arena, 0u32).is_some());
}
