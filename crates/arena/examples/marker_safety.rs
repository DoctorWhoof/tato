//! Example demonstrating type safety with and without custom markers

use tato_arena::Arena;

struct PlayerArena;
struct EnemyArena;

fn main() {
    println!("=== Marker Type Safety Demo ===\n");
    
    demonstrate_default_marker_safety();
    demonstrate_custom_marker_safety();
}

fn demonstrate_default_marker_safety() {
    println!("1. Default Marker Safety (Marker = ())");
    println!("--------------------------------------");
    
    // Both arenas use the default marker `()`
    let mut arena1: Arena<1024> = Arena::new();
    let mut arena2: Arena<1024> = Arena::new();
    
    let id1 = arena1.alloc(42u32).unwrap();
    let id2 = arena2.alloc(100u32).unwrap();
    
    println!("Arena1 generation: {}", arena1.generation());
    println!("Arena2 generation: {}", arena2.generation());
    println!("ID1 generation: {}", id1.generation());
    println!("ID2 generation: {}", id2.generation());
    
    // ✅ GOOD: These work as expected
    println!("Arena1 with ID1: {:?}", arena1.get(&id1));
    println!("Arena2 with ID2: {:?}", arena2.get(&id2));
    
    // ⚠️  PROBLEM: These compile and may return unexpected data!
    // The type system CAN'T prevent this mixing!
    // Both arenas start at generation 0, so the handles appear "valid"
    // but you get the wrong arena's data!
    println!("Arena1 with ID2 (WRONG!): {:?}", arena1.get(&id2)); // Gets arena1's data at ID2's offset!
    println!("Arena2 with ID1 (WRONG!): {:?}", arena2.get(&id1)); // Gets arena2's data at ID1's offset!
    
    // Let's demonstrate the danger by clearing one arena
    arena2.clear(); // This increments arena2's generation to 1
    println!("After clearing arena2:");
    println!("Arena1 with ID2 (now safe): {:?}", arena1.get(&id2)); // Now returns None - generation mismatch!
    println!("Arena2 with ID1 (now safe): {:?}", arena2.get(&id1)); // Now returns None - generation mismatch!
    
    println!("☑️  You get: Generational safety, bounds safety, type safety");
    println!("❌ You don't get: Cross-arena safety (mixing handles compiles and may return wrong data)\n");
}

fn demonstrate_custom_marker_safety() {
    println!("2. Custom Marker Safety");
    println!("-----------------------");
    
    // Different marker types for each arena
    let mut player_arena: Arena<1024, usize, PlayerArena> = Arena::new();
    let mut enemy_arena: Arena<1024, usize, EnemyArena> = Arena::new();
    
    let player_id = player_arena.alloc(42u32).unwrap();
    let enemy_id = enemy_arena.alloc(100u32).unwrap();
    
    // ✅ GOOD: These work as expected
    println!("Player arena with player ID: {:?}", player_arena.get(&player_id));
    println!("Enemy arena with enemy ID: {:?}", enemy_arena.get(&enemy_id));
    
    // ✅ EXCELLENT: These DON'T COMPILE due to type marker mismatch!
    // Uncomment these lines to see compile errors:
    
    // player_arena.get(&enemy_id);  // ❌ Compile error!
    // enemy_arena.get(&player_id);  // ❌ Compile error!
    
    println!("✅ You get: Generational safety, bounds safety, type safety, AND cross-arena safety!");
    println!("🎯 Custom markers prevent handle mixing at COMPILE TIME\n");
}

#[allow(dead_code)]
fn show_what_safety_you_always_get() {
    println!("3. Safety You Always Get (Regardless of Marker)");
    println!("-----------------------------------------------");
    
    let mut arena: Arena<1024> = Arena::new();
    
    // ✅ Type safety: Can't use wrong type
    let id_u32 = arena.alloc(42u32).unwrap();
    // let wrong: &u64 = arena.get(&id_u32).unwrap(); // ❌ Compile error!
    
    // ✅ Generational safety: Handles become invalid after clear/restore
    arena.clear();
    assert!(arena.get(&id_u32).is_none()); // Returns None, doesn't crash
    
    // ✅ Bounds safety: Can't access out-of-bounds memory
    // The arena checks all accesses are within allocated regions
    
    println!("Core safety features work regardless of marker type!");
}