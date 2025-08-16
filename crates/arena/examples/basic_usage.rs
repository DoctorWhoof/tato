//! This example shows the fundamental operations of the arena allocator:
//! - Allocating values
//! - Accessing values safely
//! - Creating and using pools
//! - Basic memory management

use tato_arena::Arena;

#[derive(Debug)]
struct Player {
    name: &'static str,
    health: u32,
}

fn main() {
    println!("=== Basic Arena Usage ===\n");

    // Create a 1KB arena
    let mut arena: Arena<1024> = Arena::new();

    // 1. Basic allocation
    println!("1. Single Allocations");
    println!("--------------------");

    let player_id = arena.alloc(Player { name: "Hero", health: 100 }).unwrap();

    let number_id = arena.alloc(42u32).unwrap();
    let text_id = arena.alloc("Hello Arena!").unwrap();

    // Access the data
    let player = arena.get(&player_id).unwrap();
    let number = arena.get(&number_id).unwrap();
    let text = arena.get(&text_id).unwrap();

    println!("Player:  Name: {}, Health: {}", player.name, player.health);
    println!("Number: {}", number);
    println!("Text: {}", text);

    // 2. Slice allocation (arrays)
    println!("\n2. Slice Allocations");
    println!("------------------");

    // Create a pool of 5 integers, all starting at 0
    let numbers_pool = arena.alloc_pool::<u32>(5).unwrap();

    // Create a pool with custom initialization
    let scores_pool = arena.alloc_pool_from_fn(3, |i| (i + 1) * 100).unwrap();

    // Access pools as slices
    let numbers = arena.get_pool(&numbers_pool).unwrap();
    let scores = arena.get_pool(&scores_pool).unwrap();

    println!("Numbers pool: {:?}", numbers);
    println!("Scores pool: {:?}", scores);

    // Modify pool data
    {
        let numbers_mut = arena.get_pool_mut(&numbers_pool).unwrap();
        numbers_mut[0] = 10;
        numbers_mut[1] = 20;
        numbers_mut[2] = 30;
    }

    let numbers = arena.get_pool(&numbers_pool).unwrap();
    println!("Modified numbers: {:?}", numbers);

    // 3. Memory usage
    println!("\n3. Memory Usage");
    println!("--------------");

    println!("Arena used: {} bytes", arena.used());
    println!("Arena remaining: {} bytes", arena.remaining());
    println!("Arena generation: {}", arena.generation());

    // 4. Clear and reset
    println!("\n4. Clear Arena");
    println!("-------------");

    arena.clear();
    println!("After clear:");
    println!("  Used: {} bytes", arena.used());
    println!("  Generation: {}", arena.generation());

    // Old handles are now invalid (safe!)
    match arena.get(&player_id) {
        Some(_) => println!("  Old handle still works"),
        None => println!("  ✓ Old handle safely rejected"),
    }

    // Can allocate new data
    let new_id = arena.alloc("New data after clear").unwrap();
    println!("  New allocation: {}", arena.get(&new_id).unwrap());
}
