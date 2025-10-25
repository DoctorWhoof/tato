use super::*;

#[test]
fn test_raw_id_conversion() {
    let mut arena: Arena<1024, u16> = Arena::new();

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
    let mut arena: Arena<1024, u16> = Arena::new();

    let id = arena.alloc(42u64).unwrap(); // 8 bytes
    let raw = id.raw();

    // This should work (correct size)
    let _typed_u64: ArenaId<u64, u16, ()> = raw.typed();

    // This should panic in debug mode (wrong size)
    // let _typed_u32: ArenaId<u32, usize, ()> = raw.typed(); // Would panic
}
