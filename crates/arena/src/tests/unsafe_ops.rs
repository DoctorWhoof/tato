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
fn test_unchecked_slice_access() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice_from_fn(3, |i| i as u32).unwrap();

    // Safe access
    let safe_slice = arena.get_slice(&slice).unwrap();
    assert_eq!(safe_slice, &[0, 1, 2]);

    // Unsafe unchecked access
    unsafe {
        let unsafe_slice = arena.get_slice_unchecked(&slice);
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
