//! Pool Demo - Simple example showing Pool usage
//!
//! This example demonstrates the Pool functionality for creating
//! runtime-sized collections in the arena.

use tato_arena::{Arena, Pool};

#[derive(Debug, Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Enemy {
    x: u8,
    y: u8,
    health: u8,
    speed: u8,
}

fn main() {
    // Create a 2KB arena
    let mut arena: Arena<2048> = Arena::new();
    
    println!("=== Pool Demo ===");
    println!("Arena capacity: {} bytes", 2048);
    
    // Create different sized pools based on "game configuration"
    let num_players = 4;
    let num_enemies = 20;
    let num_bullets = 100;
    
    println!("\nAllocating pools:");
    println!("- {} players", num_players);
    println!("- {} enemies", num_enemies);
    println!("- {} bullets", num_bullets);
    
    // Allocate pools
    let player_pool = arena.alloc_pool::<Point>(num_players)
        .expect("Failed to allocate player pool");
    
    let enemy_pool = arena.alloc_pool::<Enemy>(num_enemies)
        .expect("Failed to allocate enemy pool");
    
    let bullet_pool = arena.alloc_pool::<Point>(num_bullets)
        .expect("Failed to allocate bullet pool");
    
    println!("\nPool info:");
    println!("- Player pool: {} elements, {} bytes", 
             player_pool.len(), player_pool.size_bytes());
    println!("- Enemy pool: {} elements, {} bytes", 
             enemy_pool.len(), enemy_pool.size_bytes());
    println!("- Bullet pool: {} elements, {} bytes", 
             bullet_pool.len(), bullet_pool.size_bytes());
    
    // Initialize some data
    {
        let players = arena.get_pool_mut(&player_pool);
        players[0] = Point { x: 100, y: 100 };
        players[1] = Point { x: 200, y: 150 };
        players[2] = Point { x: 300, y: 200 };
        players[3] = Point { x: 400, y: 250 };
    }
    
    {
        let enemies = arena.get_pool_mut(&enemy_pool);
        for (i, enemy) in enemies.iter_mut().enumerate() {
            *enemy = Enemy {
                x: (i * 50) as u8,
                y: (i * 30) as u8,
                health: 100,
                speed: 5 + (i % 3) as u8,
            };
        }
    }
    
    {
        let bullets = arena.get_pool_mut(&bullet_pool);
        // Initialize first 10 bullets
        for i in 0..10 {
            bullets[i] = Point { 
                x: i as i32 * 10, 
                y: 500 
            };
        }
    }
    
    // Read back and display some data
    println!("\nSample data:");
    
    let players = arena.get_pool(&player_pool);
    println!("Players:");
    for (i, player) in players.iter().enumerate() {
        println!("  Player {}: {:?}", i, player);
    }
    
    let enemies = arena.get_pool(&enemy_pool);
    println!("First 5 enemies:");
    for (i, enemy) in enemies.iter().take(5).enumerate() {
        println!("  Enemy {}: {:?}", i, enemy);
    }
    
    let bullets = arena.get_pool(&bullet_pool);
    println!("Active bullets:");
    for (i, bullet) in bullets.iter().take(10).enumerate() {
        println!("  Bullet {}: {:?}", i, bullet);
    }
    
    // Show memory usage
    println!("\nMemory usage:");
    println!("- Used: {} bytes", arena.used());
    println!("- Remaining: {} bytes", arena.remaining());
    println!("- Usage: {:.1}%", (arena.used() as f32 / 2048.0) * 100.0);
    println!("- Allocations: {}", arena.allocation_count());
    
    // Demonstrate slice operations
    println!("\nSlice operations:");
    let player_slice = arena.get_pool(&player_pool);
    println!("- Player pool length: {}", player_slice.len());
    println!("- First player: {:?}", player_slice.first());
    println!("- Last player: {:?}", player_slice.last());
    
    // Show different pool sizes work
    println!("\nDynamic sizing demo:");
    let small_pool = arena.alloc_pool::<u32>(3).expect("Small pool");
    let large_pool = arena.alloc_pool::<u32>(50).expect("Large pool");
    
    println!("- Small pool: {} elements", small_pool.len());
    println!("- Large pool: {} elements", large_pool.len());
    
    // Fill the large pool with a pattern
    {
        let large_slice = arena.get_pool_mut(&large_pool);
        for (i, val) in large_slice.iter_mut().enumerate() {
            *val = i as u32 * i as u32; // Square numbers
        }
    }
    
    // Show pattern
    let large_slice = arena.get_pool(&large_pool);
    print!("Square numbers: ");
    for val in large_slice.iter().take(10) {
        print!("{} ", val);
    }
    println!("...");
    
    // Final memory stats
    println!("\nFinal memory usage:");
    println!("- Used: {} bytes", arena.used());
    println!("- Remaining: {} bytes", arena.remaining());
    println!("- Usage: {:.1}%", (arena.used() as f32 / 2048.0) * 100.0);
    
    println!("\n=== Pool Demo Complete ===");
}