//! Example showing the cleaner API with ArenaIndex trait

use tato_arena::{Arena, ArenaIndex};

// Custom index type that implements ArenaIndex
// (This would work if we added the necessary trait implementations)
// struct MyIndex(u16);

fn demonstrate_clean_signatures() {
    println!("=== Clean API Demo ===");

    // Before: Arena<1024, SizeType> where SizeType: Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = SizeType>
    // After: Arena<1024, SizeType> where SizeType: ArenaIndex

    // These all work with the same clean constraint
    let mut arena_u8: Arena<256, u8> = Arena::new();
    let mut arena_u16: Arena<65536, u16> = Arena::new();
    let mut arena_usize: Arena<1024, usize> = Arena::new();

    // Allocate some data
    let id1 = arena_u8.alloc(42u32).unwrap();
    let id2 = arena_u16.alloc("Hello".to_string()).unwrap();
    let id3 = arena_usize.alloc([1, 2, 3, 4]).unwrap();

    // Access data
    println!("u8 arena: {}", arena_u8.get(&id1).unwrap());
    println!("u16 arena: {}", arena_u16.get(&id2).unwrap());
    println!("usize arena: {:?}", arena_usize.get(&id3).unwrap());

    // Create pools
    let pool1 = arena_u8.alloc_pool::<i32>(10).unwrap();
    let pool2 = arena_u16.alloc_pool_from_fn(5, |i| i as f32).unwrap();
    let pool3 = arena_usize.alloc_pool::<bool>(8).unwrap();

    println!("Pool lengths: {}, {}, {}",
             pool1.len(), pool2.len(), pool3.len());

    // Show memory usage
    println!("Memory usage:");
    println!("  u8 arena: {}/{} bytes", arena_u8.used(), 256);
    println!("  u16 arena: {}/{} bytes", arena_u16.used(), 65536);
    println!("  usize arena: {}/{} bytes", arena_usize.used(), 1024);
}

fn show_trait_benefits() {
    println!("\n=== ArenaIndex Trait Benefits ===");

    // The ArenaIndex trait consolidates these requirements:
    // - Copy: Can be copied cheaply
    // - TryFrom<usize>: Can be created from usize (with potential failure)
    // - Into<usize>: Can be converted to usize (always succeeds for our types)
    // - PartialOrd: Can be compared for ordering
    // - Add<Output = Self>: Can be added together
    // - tato_math::Num: Provides zero() and other numeric operations

    println!("Supported index types:");
    println!("  u8: 0-255 bytes (great for tiny arenas)");
    println!("  u16: 0-65,535 bytes (good for small-medium arenas)");
    println!("  usize: 0-{} bytes (full system capacity)", usize::MAX);

    // Show the memory savings
    use core::mem::size_of;
    println!("\nHandle sizes:");
    println!("  ArenaId<T, u8>: {} bytes", size_of::<tato_arena::ArenaId<u32, u8>>());
    println!("  ArenaId<T, u16>: {} bytes", size_of::<tato_arena::ArenaId<u32, u16>>());
    println!("  ArenaId<T, usize>: {} bytes", size_of::<tato_arena::ArenaId<u32, usize>>());
}

// Example of a generic function that works with any ArenaIndex
fn generic_arena_function<I: ArenaIndex>(arena: &mut Arena<1024, I>) -> Option<I> {
    // This function works with any valid index type
    let _id = arena.alloc(123u32)?;
    Some(arena.used().try_into().ok()?)
}

fn main() {
    demonstrate_clean_signatures();
    show_trait_benefits();

    println!("\n=== Generic Functions ===");

    let mut arena_u16: Arena<1024, u16> = Arena::new();
    let mut arena_usize: Arena<1024, usize> = Arena::new();

    if let Some(used) = generic_arena_function(&mut arena_u16) {
        println!("u16 arena used: {} bytes", used);
    }

    if let Some(used) = generic_arena_function(&mut arena_usize) {
        println!("usize arena used: {} bytes", used);
    }

    println!("\n✓ Much cleaner than the old verbose type constraints!");
}
