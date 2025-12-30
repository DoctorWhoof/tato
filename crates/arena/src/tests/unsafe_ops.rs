use super::*;

#[test]
fn test_slice_access() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = arena.alloc_slice_from_fn(3, |i| i as u32).unwrap();

    // Safe access
    let safe_slice = arena.get_slice(slice).unwrap();
    assert_eq!(safe_slice, &[0, 1, 2]);
}