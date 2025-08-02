//! Example demonstrating generational safety in the arena allocator.
//! This shows how the arena prevents use-after-restore bugs by invalidating
//! handles when the arena is restored to a previous state.

use tato_arena::Arena;

#[derive(Debug, Clone)]
struct Player {
    name: String,
    health: u32,
}

#[derive(Debug, Clone)]
struct Enemy {
    damage: u32,
    speed: u32,
}

// Type markers for different arena instances
struct GameArena;
struct TempArena;

fn main() {
    println!("=== Generational Arena Safety Demo ===\n");

    // Demonstrate basic generational safety
    basic_generational_safety();

    // Demonstrate restore_to safety
    restore_to_safety();

    // Demonstrate type marker safety
    type_marker_safety();

    // Demonstrate pool safety
    pool_safety();
}

fn basic_generational_safety() {
    println!("1. Basic Generational Safety");
    println!("----------------------------");

    let mut arena: Arena<1024> = Arena::new();

    // Allocate some data
    let player_id = arena.alloc(Player {
        name: "Hero".to_string(),
        health: 100,
    }).unwrap();

    println!("Initial generation: {}", arena.generation());
    println!("Player ID generation: {}", player_id.generation());

    // Access should work
    match arena.get(&player_id) {
        Some(player) => println!("Player: {} (Health: {})", player.name, player.health),
        None => println!("Player not found!"),
    }

    // Clear arena (increments generation)
    arena.clear();
    println!("\nAfter clear:");
    println!("Arena generation: {}", arena.generation());
    println!("Player ID generation: {}", player_id.generation());

    // Access should fail due to generation mismatch
    match arena.get(&player_id) {
        Some(player) => println!("Player: {} (Health: {})", player.name, player.health),
        None => println!("Player not found! (Handle is stale)"),
    }

    // New allocations work with new generation
    let new_player_id = arena.alloc(Player {
        name: "New Hero".to_string(),
        health: 150,
    }).unwrap();

    println!("New player ID generation: {}", new_player_id.generation());
    match arena.get(&new_player_id) {
        Some(player) => println!("New player: {} (Health: {})", player.name, player.health),
        None => println!("New player not found!"),
    }

    println!();
}

fn restore_to_safety() {
    println!("2. Restore-to Safety");
    println!("--------------------");

    let mut arena: Arena<1024> = Arena::new();

    // Phase 1: Allocate initial data
    let player1_id = arena.alloc(Player {
        name: "Player1".to_string(),
        health: 100,
    }).unwrap();

    let checkpoint = arena.used();
    println!("Checkpoint at {} bytes", checkpoint);

    // Phase 2: Allocate more data
    let player2_id = arena.alloc(Player {
        name: "Player2".to_string(),
        health: 80,
    }).unwrap();

    let enemy_id = arena.alloc(Enemy {
        damage: 25,
        speed: 10,
    }).unwrap();

    println!("Before restore - Arena generation: {}", arena.generation());

    // Both should be accessible
    println!("Player1: {:?}", arena.get(&player1_id).map(|p| &p.name));
    println!("Player2: {:?}", arena.get(&player2_id).map(|p| &p.name));
    println!("Enemy damage: {:?}", arena.get(&enemy_id).map(|e| e.damage));

    // Restore to checkpoint (this invalidates ALL handles)
    arena.restore_to(checkpoint);
    println!("\nRestored to checkpoint");
    println!("After restore - Arena generation: {}", arena.generation());

    // ALL handles should now be invalid (generation changed)
    println!("Player1: {:?}", arena.get(&player1_id).map(|p| &p.name));
    println!("Player2: {:?}", arena.get(&player2_id).map(|p| &p.name));
    println!("Enemy damage: {:?}", arena.get(&enemy_id).map(|e| e.damage));

    // Can allocate new data in the restored space
    let new_enemy_id = arena.alloc(Enemy {
        damage: 40,
        speed: 15,
    }).unwrap();

    println!("New enemy (generation {}): damage = {:?}",
             new_enemy_id.generation(),
             arena.get(&new_enemy_id).map(|e| e.damage));

    println!();
}

fn type_marker_safety() {
    println!("3. Type Marker Safety");
    println!("---------------------");

    let mut game_arena: Arena<1024, usize, GameArena> = Arena::new();
    let mut temp_arena: Arena<1024, usize, TempArena> = Arena::new();

    let game_player = game_arena.alloc(Player {
        name: "Game Player".to_string(),
        health: 100,
    }).unwrap();

    let temp_player = temp_arena.alloc(Player {
        name: "Temp Player".to_string(),
        health: 50,
    }).unwrap();

    // These work fine
    println!("Game player: {:?}", game_arena.get(&game_player).map(|p| &p.name));
    println!("Temp player: {:?}", temp_arena.get(&temp_player).map(|p| &p.name));

    // These would cause compile-time errors due to type marker mismatch:
    // game_arena.get(&temp_player); // ERROR!
    // temp_arena.get(&game_player); // ERROR!

    println!("✓ Type markers prevent cross-arena access at compile time");
    println!();
}

fn pool_safety() {
    println!("4. Pool Generational Safety");
    println!("---------------------------");

    let mut arena: Arena<2048> = Arena::new();

    // Create a pool of enemies
    let enemy_pool = arena.alloc_pool_from_fn(5, |i| Enemy {
        damage: 10 + (i as u32 * 5),
        speed: 5 + (i as u32 * 2),
    }).unwrap();

    println!("Enemy pool generation: {}", enemy_pool.generation());

    // Access pool
    if let Some(enemies) = arena.get_pool(&enemy_pool) {
        println!("Enemies in pool:");
        for (i, enemy) in enemies.iter().enumerate() {
            println!("  Enemy {}: damage={}, speed={}", i, enemy.damage, enemy.speed);
        }
    }

    // Clear arena
    arena.clear();
    println!("\nAfter clearing arena:");
    println!("Arena generation: {}", arena.generation());
    println!("Pool generation: {}", enemy_pool.generation());

    // Pool access should fail
    match arena.get_pool(&enemy_pool) {
        Some(_) => println!("Pool is still accessible"),
        None => println!("✓ Pool is no longer accessible (stale handle)"),
    }

    // Create new pool with new generation
    let new_pool = arena.alloc_pool_from_fn(3, |i| Enemy {
        damage: 20 + (i as u32 * 10),
        speed: 8 + (i as u32 * 3),
    }).unwrap();

    println!("New pool generation: {}", new_pool.generation());

    if let Some(enemies) = arena.get_pool(&new_pool) {
        println!("New enemies in pool:");
        for (i, enemy) in enemies.iter().enumerate() {
            println!("  Enemy {}: damage={}, speed={}", i, enemy.damage, enemy.speed);
        }
    }

    println!();
}
