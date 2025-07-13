//! Example demonstrating the use of smaller index types in tato_arena
//!
//! This shows how you can use u8, u16, or u32 instead of usize for arena indices,
//! significantly reducing memory usage when you have many handles to arena-allocated objects.

#![allow(unused)]

use tato_arena::Arena;
use core::mem::size_of_val;

fn main() {
    demonstrate_index_sizes();
    practical_example();
}

fn demonstrate_index_sizes() {
    println!("=== Index Type Size Comparison ===\n");

    // Standard arena with usize indices (default)
    let mut arena_usize: Arena<1024, usize> = Arena::new();
    let id_usize = arena_usize.alloc(42u32).unwrap();

    // Arena with u16 indices (for up to 64KB)
    let mut arena_u16: Arena<1024, u16> = Arena::new();
    let id_u16 = arena_u16.alloc(42u32).unwrap();

    // Arena with u8 indices (for up to 256 bytes)
    let mut arena_u8: Arena<256, u8> = Arena::new();
    let id_u8 = arena_u8.alloc(42u32).unwrap();

    println!("ArenaId<u32, usize> size: {} bytes", size_of_val(&id_usize));
    println!("ArenaId<u32, u16> size: {} bytes", size_of_val(&id_u16));
    println!("ArenaId<u32, u8> size: {} bytes", size_of_val(&id_u8));
    println!();

    // On 64-bit systems:
    // - ArenaId<T, usize> is 16 bytes (8 + 8)
    // - ArenaId<T, u16> is 4 bytes (2 + 2)
    // - ArenaId<T, u8> is 2 bytes (1 + 1)

    println!("Memory savings:");
    println!("- u16 vs usize: {}x smaller", size_of_val(&id_usize) / size_of_val(&id_u16));
    println!("- u8 vs usize: {}x smaller", size_of_val(&id_usize) / size_of_val(&id_u8));
}

fn practical_example() {
    println!("\n=== Practical Example: Game Entity System ===\n");

    // For a small game with entities that fit in 64KB
    let mut entity_arena: Arena<65536, u16> = Arena::new();

    // Define some game components
    #[derive(Debug)]
    struct Position { x: f32, y: f32 }

    #[derive(Debug)]
    struct Velocity { dx: f32, dy: f32 }

    #[derive(Debug)]
    struct Health { current: u32, max: u32 }

    // Create some entities
    let player_pos = entity_arena.alloc(Position { x: 100.0, y: 200.0 }).unwrap();
    let player_vel = entity_arena.alloc(Velocity { dx: 0.0, dy: 0.0 }).unwrap();
    let player_health = entity_arena.alloc(Health { current: 100, max: 100 }).unwrap();

    let enemy_pos = entity_arena.alloc(Position { x: 500.0, y: 300.0 }).unwrap();
    let enemy_health = entity_arena.alloc(Health { current: 50, max: 50 }).unwrap();

    // Store entity handles in a compact structure
    struct Entity {
        position: Option<tato_arena::ArenaId<Position, u16>>,
        velocity: Option<tato_arena::ArenaId<Velocity, u16>>,
        health: Option<tato_arena::ArenaId<Health, u16>>,
    }

    let player = Entity {
        position: Some(player_pos),
        velocity: Some(player_vel),
        health: Some(player_health),
    };

    let enemy = Entity {
        position: Some(enemy_pos),
        velocity: None,  // Static enemy
        health: Some(enemy_health),
    };

    // Memory savings calculation
    let entity_size = size_of_val(&player);
    let ptr_size = size_of::<Option<tato_arena::ArenaId<Position, u16>>>();
    let usize_ptr_size = size_of::<Option<tato_arena::ArenaId<Position, usize>>>();

    println!("Entity struct size: {} bytes", entity_size);
    println!("Each Option<ArenaId<T, u16>> is {} bytes", ptr_size);
    println!("(Would be {} bytes each with usize indices)", usize_ptr_size);

    // Access components
    if let Some(pos_id) = player.position {
        let pos = entity_arena.get(&pos_id);
        println!("\nPlayer position: {:?}", pos);
    }

    if let Some(health_id) = enemy.health {
        let health = entity_arena.get(&health_id);
        println!("Enemy health: {:?}", health);
    }

    println!("\nArena stats:");
    println!("- Used: {} bytes", entity_arena.used());
    println!("- Remaining: {} bytes", entity_arena.remaining());
    println!("- Allocations: {}", entity_arena.allocation_count());
}
