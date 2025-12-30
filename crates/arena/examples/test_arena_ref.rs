use tato_arena::{Arena, ArenaRef, ArenaOps};

fn main() {
    // Create an arena with a specific size
    let mut arena: Arena<1024, u32> = Arena::new();
    
    // Test basic arena operations work
    let id1 = arena.alloc(42u32).unwrap();
    let value1 = arena.get(id1).unwrap();
    assert_eq!(*value1, 42);
    println!("Direct arena allocation works: {}", *value1);
    
    // Now test with ArenaRef
    let mut arena_ref = arena.as_ref();
    
    // Test allocation through ArenaRef
    let id2 = arena_ref.alloc(100u32).unwrap();
    let value2 = arena_ref.get(id2).unwrap();
    assert_eq!(*value2, 100);
    println!("ArenaRef allocation works: {}", *value2);
    
    // Test slice allocation through ArenaRef
    let data = vec![1u32, 2, 3, 4, 5];
    let slice_id = arena_ref.alloc_slice(&data).unwrap();
    let slice = arena_ref.get_slice(slice_id).unwrap();
    assert_eq!(slice, &[1, 2, 3, 4, 5]);
    println!("ArenaRef slice allocation works: {:?}", slice);
    
    // Test that we can pass ArenaRef to a function that doesn't know the size
    test_with_size_erased_ref(arena_ref);
    
    println!("\nAll tests passed!");
}

// This function accepts any ArenaRef, regardless of the original arena size
fn test_with_size_erased_ref<I, M>(mut arena_ref: ArenaRef<'_, I, M>)
where
    I: tato_arena::ArenaIndex,
    M: Copy,
{
    println!("\n--- Testing with size-erased reference ---");
    
    // The function doesn't know the arena size (LEN parameter is gone)
    // but can still use all arena operations
    
    let id = arena_ref.alloc(999u64).unwrap();
    let value = arena_ref.get(id).unwrap();
    assert_eq!(*value, 999);
    println!("Size-erased allocation works: {}", *value);
    
    // Check arena statistics
    println!("Arena stats:");
    println!("  Used: {} bytes", arena_ref.used());
    println!("  Remaining: {} bytes", arena_ref.remaining());
    println!("  Capacity: {} bytes", arena_ref.capacity());
    println!("  Generation: {}", arena_ref.generation());
    println!("  Arena ID: {}", arena_ref.arena_id());
}