//! This example shows how to use smaller index types to reduce memory usage
//! of arena handles, which is especially important when you have many handles
//! stored in your application.

use tato_arena::Arena;

#[derive(Debug, Default)]
struct Item {
    value: u32,
    active: bool,
}

fn main() {
    println!("=== Memory Optimization ===\n");

    demonstrate_handle_sizes();
    demonstrate_capacity_limits();
    demonstrate_practical_usage();
}

fn demonstrate_handle_sizes() {
    println!("1. Handle Size Comparison");
    println!("------------------------");

    // Different arena types with different index sizes
    let mut arena_default: Arena<4096> = Arena::new(); // Default u16 (2 bytes) - RECOMMENDED!
    let mut arena_usize: Arena<4096, usize> = Arena::new(); // Large (8 bytes on 64-bit)
    let mut arena_u8: Arena<255, u8> = Arena::new(); // Tiny (1 byte)

    // Allocate the same data in each
    let id_default = arena_default.alloc(Item { value: 100, active: true }).unwrap();
    let id_usize = arena_usize.alloc(Item { value: 200, active: true }).unwrap();
    let id_u8 = arena_u8.alloc(Item { value: 300, active: true }).unwrap();

    // Show handle sizes
    use core::mem::size_of_val;
    println!("Handle sizes:");
    println!("  Default (u16): {} bytes per handle ⭐ RECOMMENDED", size_of_val(&id_default));
    println!("  usize arena:   {} bytes per handle", size_of_val(&id_usize));
    println!("  u8 arena:      {} bytes per handle", size_of_val(&id_u8));

    // Explain the memory optimization
    println!("\n💡 Memory Optimization Results:");
    println!("  Default u16: 8-byte handles (perfect for most apps!)");
    println!("  u8 arena:    6-byte handles (50% smaller, for tiny arenas)");
    println!("  usize arena: 24-byte handles (use only if you need >64KB)");

    // All work the same way
    println!("All handles work identically:");
    let item_default = arena_default.get(&id_default).unwrap();
    let item_usize = arena_usize.get(&id_usize).unwrap();
    let item_u8 = arena_u8.get(&id_u8).unwrap();

    println!("  Default: value={}, active={}", item_default.value, item_default.active);
    println!("  usize:   value={}, active={}", item_usize.value, item_usize.active);
    println!("  u8:      value={}, active={}", item_u8.value, item_u8.active);
    println!();
}

fn demonstrate_capacity_limits() {
    println!("2. Capacity Limits");
    println!("-----------------");

    // Different index types have different capacity limits
    println!("Arena capacity limits:");
    println!("  u8:      up to 255 bytes");
    println!("  u16:     up to 65,535 bytes (64KB) ⭐ DEFAULT");
    println!("  usize:   up to {} bytes", usize::MAX);

    // Demonstrate u8 arena reaching its limit
    let mut small_arena: Arena<200, u8> = Arena::new();
    let mut count = 0;

    // Fill the arena
    while let Some(_) = small_arena.alloc(Item::default()) {
        count += 1;
        if count > 50 {
            break;
        } // Safety limit for demo
    }

    println!("\nu8 arena demonstration:");
    println!("  Allocated {} items", count);
    println!("  Used {}/{} bytes", small_arena.used(), 200);
    println!("  Remaining: {} bytes", small_arena.remaining());
    println!();
}

fn demonstrate_practical_usage() {
    println!("3. Practical Usage");
    println!("-----------------");

    // Simulate a game with many entities
    let mut game_arena: Arena<32768> = Arena::new(); // 32KB with default u16 indices

    // Create slices of game objects
    let players_slice = game_arena.alloc_slice::<Item>(64).unwrap();
    let enemies_slice = game_arena
        .alloc_slice_from_fn(128, |i| Item { value: i as u32 * 10, active: i % 3 == 0 })
        .unwrap();

    println!("Game arena usage:");
    println!("  Arena capacity: 32KB with default u16 indices");
    println!("  Players slice: {} items", players_slice.len());
    println!("  Enemies slice: {} items", enemies_slice.len());
    println!("  Memory used: {} bytes", game_arena.used());

    // Show memory savings calculation
    let handles_stored = 192; // 64 players + 128 enemies
    let usize_handle_size = 24; // Size with usize indices
    let u16_handle_size = 8; // Size with u16 indices (DEFAULT)

    let usize_total = handles_stored * usize_handle_size;
    let u16_total = handles_stored * u16_handle_size;
    let savings = usize_total - u16_total;

    println!("\nMemory savings calculation:");
    println!("  {} handles stored in game:", handles_stored);
    println!("  With usize: {} bytes", usize_total);
    println!("  With u16:   {} bytes (DEFAULT)", u16_total);
    println!(
        "  Savings:    {} bytes ({:.1}x smaller)",
        savings,
        usize_total as f32 / u16_total as f32
    );
    println!("  🎉 u16 default gives you 64KB capacity with 3x smaller handles!");

    // Access some data to show it works normally
    if let Some(enemies) = game_arena.get_slice(&enemies_slice) {
        println!("\nSample enemies:");
        for (i, enemy) in enemies.iter().take(5).enumerate() {
            println!("  Enemy {}: value={}, active={}", i, enemy.value, enemy.active);
        }
    }

    // Show that players slice is also accessible
    if let Some(players) = game_arena.get_slice(&players_slice) {
        println!(
            "Players slice has {} slots, first player active: {}",
            players.len(),
            players.first().map(|p| p.active).unwrap_or(false)
        );
    }

    println!("\n💡 Choose your index type based on arena size:");
    println!("   DEFAULT:  Arena<SIZE>           (u16, 8-byte handles, up to 64KB) ⭐");
    println!("   TINY:     Arena<SIZE, u8>       (6-byte handles, up to 255 bytes)");
    println!("   HUGE:     Arena<SIZE, usize>    (24-byte handles, unlimited size)");
    println!("   ✨ u16 default: 65k arenas, 65k generations - perfect for most apps!");
}
