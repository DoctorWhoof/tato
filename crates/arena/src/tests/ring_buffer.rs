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
fn test_ring_buffer_try_push_when_full() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 2).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();

    assert!(buffer.is_full());

    // try_push should fail when full
    assert!(buffer.try_push(&mut arena, 30).is_err());
    assert_eq!(buffer.len(), 2);
}

#[test]
fn test_ring_buffer_push_auto_overwrite() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();

    // Fill buffer
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    assert!(buffer.is_full());

    // Push should automatically overwrite oldest element
    buffer.push(&mut arena, 40).unwrap();

    assert!(buffer.is_full());
    assert_eq!(buffer.len(), 3);

    // First element should be 20 (10 was overwritten)
    assert_eq!(buffer.pop(&arena), Some(20));
    assert_eq!(buffer.pop(&arena), Some(30));
    assert_eq!(buffer.pop(&arena), Some(40));
}

#[test]
fn test_ring_buffer_push_multiple_overwrites() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 2).unwrap();

    // Fill buffer
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();

    // Push multiple times - should automatically overwrite
    buffer.push(&mut arena, 30).unwrap(); // overwrites 10
    buffer.push(&mut arena, 40).unwrap(); // overwrites 20
    buffer.push(&mut arena, 50).unwrap(); // overwrites 30

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

    // Push more elements (buffer has space now)
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

    // Push should overwrite automatically
    buffer.push(&mut arena, 99).unwrap();
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
        buffer.push(&mut arena, i).unwrap(); // Auto-overwrite when full

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

#[test]
fn test_ring_buffer_iterator_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let buffer = RingBuffer::<i32>::new(&mut arena, 5).unwrap();
    
    let mut iter = buffer.items(&arena);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.len(), 0);
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn test_ring_buffer_iterator_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 5).unwrap();
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    let mut iter = buffer.items(&arena);
    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));
    
    // Should iterate in FIFO order
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next(), Some(&20));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_ring_buffer_iterator_after_wrap() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();
    
    // Fill buffer
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    
    // Overwrite first element
    buffer.push(&mut arena, 4).unwrap();
    
    // Iterator should give logical order: [2, 3, 4]
    let mut iter = buffer.items(&arena);
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&4));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_ring_buffer_iterator_after_pop() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 4).unwrap();
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    buffer.push(&mut arena, 40).unwrap();
    
    // Pop two elements
    buffer.pop(&arena);
    buffer.pop(&arena);
    
    // Iterator should show remaining elements in order
    let mut iter = buffer.items(&arena);
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), Some(&40));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_ring_buffer_iterator_collect_pattern() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 4).unwrap();
    
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    
    // Pop one, push two more (causing wrap)
    buffer.pop(&arena);
    buffer.push(&mut arena, 4).unwrap();
    buffer.push(&mut arena, 5).unwrap();
    
    // Should contain [2, 3, 4, 5] in logical order
    let sum: i32 = buffer.items(&arena).map(|&x| x).sum();
    assert_eq!(sum, 2 + 3 + 4 + 5);
    
    let count = buffer.items(&arena).count();
    assert_eq!(count, 4);
}

#[test]
fn test_ring_buffer_iterator_reverse() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 4).unwrap();
    
    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();
    
    // Forward iteration
    let mut forward_iter = buffer.items(&arena);
    assert_eq!(forward_iter.next(), Some(&10));
    assert_eq!(forward_iter.next(), Some(&20));
    assert_eq!(forward_iter.next(), Some(&30));
    assert_eq!(forward_iter.next(), None);
    
    // Reverse iteration
    let mut reverse_iter = buffer.items(&arena).rev();
    assert_eq!(reverse_iter.next(), Some(&30));
    assert_eq!(reverse_iter.next(), Some(&20));
    assert_eq!(reverse_iter.next(), Some(&10));
    assert_eq!(reverse_iter.next(), None);
}

#[test]
fn test_ring_buffer_iterator_reverse_after_wrap() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();
    
    // Fill and wrap
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    buffer.push(&mut arena, 4).unwrap(); // wraps, buffer now has [2, 3, 4]
    
    // Reverse iteration should be [4, 3, 2]
    let mut reverse_iter = buffer.items(&arena).rev();
    assert_eq!(reverse_iter.next(), Some(&4));
    assert_eq!(reverse_iter.next(), Some(&3));
    assert_eq!(reverse_iter.next(), Some(&2));
    assert_eq!(reverse_iter.next(), None);
}

#[test]
fn test_ring_buffer_iterator_double_ended() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 5).unwrap();
    
    buffer.push(&mut arena, 1).unwrap();
    buffer.push(&mut arena, 2).unwrap();
    buffer.push(&mut arena, 3).unwrap();
    buffer.push(&mut arena, 4).unwrap();
    buffer.push(&mut arena, 5).unwrap();
    
    let mut iter = buffer.items(&arena);
    
    // Mix forward and backward iteration
    assert_eq!(iter.next(), Some(&1));        // front
    assert_eq!(iter.next_back(), Some(&5));   // back
    assert_eq!(iter.next(), Some(&2));        // front
    assert_eq!(iter.next_back(), Some(&4));   // back
    assert_eq!(iter.next(), Some(&3));        // front (middle)
    assert_eq!(iter.next(), None);            // exhausted
    assert_eq!(iter.next_back(), None);       // exhausted
}

#[test]
fn test_ring_buffer_iterator_rev_simple() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = RingBuffer::new(&mut arena, 3).unwrap();
    
    buffer.push(&mut arena, 100).unwrap();
    buffer.push(&mut arena, 200).unwrap();
    buffer.push(&mut arena, 300).unwrap();
    
    // Test that .rev() works
    // Test that .rev() works
    let mut iter = buffer.items(&arena).rev();
    let first = iter.next().unwrap();
    let second = iter.next().unwrap(); 
    let third = iter.next().unwrap();
    
    assert_eq!(*first, 300);
    assert_eq!(*second, 200);
    assert_eq!(*third, 100);
    assert_eq!(iter.next(), None);
}
