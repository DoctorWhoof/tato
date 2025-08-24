use super::*;
use crate::Buffer;

#[test]
fn test_arena_iter_slice_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let empty_slice = arena.alloc_slice::<u32>(0).unwrap();
    let iter = arena.iter_slice(&empty_slice).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_arena_iter_slice_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32 * 10).unwrap();

    let mut iter = arena.iter_slice(&slice).unwrap();
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
    assert!(arena.iter_slice(&slice).is_err());
}

#[test]
fn test_arena_iter_slice_range_full() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    let iter = arena.iter_slice_range(&slice, 0, 5).unwrap();
    assert_eq!(iter.count(), 5);
}

#[test]
fn test_arena_iter_slice_range_partial() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(10, |i| i as u32).unwrap();

    let mut iter = arena.iter_slice_range(&slice, 2, 5).unwrap();
    assert_eq!(iter.next(), Some(&2));
    assert_eq!(iter.next(), Some(&3));
    assert_eq!(iter.next(), Some(&4));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_arena_iter_slice_range_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    let iter = arena.iter_slice_range(&slice, 3, 3).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_arena_iter_slice_range_out_of_bounds() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(10, |i| i as u32).unwrap();

    // End beyond slice length
    assert!(arena.iter_slice_range(&slice, 5, 15).is_err());

    // Start > end
    assert!(arena.iter_slice_range(&slice, 8, 5).is_err());
}

#[test]
fn test_arena_iter_slice_range_invalid_after_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(5, |i| i as u32).unwrap();

    arena.clear();
    assert!(arena.iter_slice_range(&slice, 0, 3).is_err());
}

#[test]
fn test_buffer_iter_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let buffer = Buffer::<i32>::new(&mut arena, 5).unwrap();

    let iter = buffer.items(&arena).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_buffer_iter_with_elements() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 10).unwrap();
    buffer.push(&mut arena, 20).unwrap();
    buffer.push(&mut arena, 30).unwrap();

    let mut iter = buffer.items(&arena).unwrap();
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next(), Some(&20));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), None);

    // Verify it only iterates over used elements, not full capacity
    assert_eq!(buffer.items(&arena).unwrap().count(), 3);
}

#[test]
fn test_buffer_iter_after_clear() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::new(&mut arena, 5).unwrap();

    buffer.push(&mut arena, 42).unwrap();
    buffer.clear();

    let iter = buffer.items(&arena).unwrap();
    assert_eq!(iter.count(), 0);
}

#[test]
fn test_iter_enumerate_pattern() {
    let mut arena: Arena<1024> = Arena::new();
    let slice = arena.alloc_slice_from_fn(4, |i| (i + 1) * 100).unwrap();

    for (i, &value) in arena.iter_slice(&slice).unwrap().enumerate() {
        assert_eq!(value, (i + 1) * 100);
    }
}

#[test]
fn test_buffer_drain_empty() {
    let mut arena: Arena<1024> = Arena::new();
    let mut buffer = Buffer::<i32>::new(&mut arena, 5).unwrap();

    let mut drain = buffer.drain(&arena);
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

    let mut drain = buffer.drain(&arena);
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

    let drain = buffer.drain(&arena);
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

    let drain = buffer.drain(&arena);
    let mut sum = 0;
    for value in drain {
        sum += value;
    }
    assert_eq!(sum, 600);
    assert_eq!(buffer.len(), 0);
}
