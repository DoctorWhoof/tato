//! Demonstration of using smaller index types to save memory

use tato_arena::Arena;

#[derive(Debug, Default)]
struct Player {
    health: u8,
    score: u16,
}

fn main() {
    println!("=== Smaller Indices Demo ===");
    
    // Standard arena with usize indices (8 bytes on 64-bit)
    let mut arena_usize: Arena<1024, usize> = Arena::new();
    
    // Arena with u16 indices (2 bytes) - good for up to 64KB
    let mut arena_u16: Arena<1024, u16> = Arena::new();
    
    // Arena with u8 indices (1 byte) - good for up to 255 bytes
    let mut arena_u8: Arena<255, u8> = Arena::new();
    
    // Allocate same data in each arena
    let id_usize = arena_usize.alloc(Player { health: 100, score: 1000 }).unwrap();
    let id_u16 = arena_u16.alloc(Player { health: 90, score: 2000 }).unwrap();
    let id_u8 = arena_u8.alloc(Player { health: 80, score: 3000 }).unwrap();
    
    // Show memory savings
    println!("ArenaId sizes:");
    println!("  usize: {} bytes", core::mem::size_of_val(&id_usize));
    println!("  u16:   {} bytes", core::mem::size_of_val(&id_u16));
    println!("  u8:    {} bytes", core::mem::size_of_val(&id_u8));
    
    // Access the data
    if let Some(player) = arena_usize.get(&id_usize) {
        println!("usize arena player: health={}, score={}", player.health, player.score);
    }
    
    if let Some(player) = arena_u16.get(&id_u16) {
        println!("u16 arena player: health={}, score={}", player.health, player.score);
    }
    
    if let Some(player) = arena_u8.get(&id_u8) {
        println!("u8 arena player: health={}, score={}", player.health, player.score);
    }
    
    // Show arena usage
    println!("\nArena usage:");
    println!("  usize arena: {} bytes", arena_usize.used());
    println!("  u16 arena:   {} bytes", arena_u16.used());
    println!("  u8 arena:    {} bytes", arena_u8.used());
    
    // Demonstrate u8 arena limits
    println!("\nTesting u8 arena capacity:");
    let mut count = 0;
    while arena_u8.alloc(Player::default()).is_some() {
        count += 1;
        if count > 100 { break; } // Safety limit
    }
    println!("u8 arena fit {} more players before running out of space", count);
    println!("Final u8 arena usage: {}/255 bytes", arena_u8.used());
}