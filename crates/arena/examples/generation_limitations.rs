//! Example demonstrating why generational indices don't provide cross-arena safety

use tato_arena::Arena;

fn main() {
    println!("=== Generational Index Limitations ===\n");

    demonstrate_generation_independence();
    demonstrate_false_security();
    demonstrate_real_world_scenario();
}

fn demonstrate_generation_independence() {
    println!("1. Each Arena Has Independent Generations");
    println!("----------------------------------------");

    let mut arena1: Arena<1024> = Arena::new();
    let mut arena2: Arena<1024> = Arena::new();

    println!("Initial state:");
    println!("  Arena1 generation: {}", arena1.generation());
    println!("  Arena2 generation: {}", arena2.generation());

    // Both start at generation 0
    let id1 = arena1.alloc("Arena1 data").unwrap();
    let id2 = arena2.alloc("Arena2 data").unwrap();

    println!("\nAfter allocations:");
    println!("  ID1 generation: {} (from arena1)", id1.generation());
    println!("  ID2 generation: {} (from arena2)", id2.generation());

    // Clear arena1 multiple times to advance its generation
    arena1.clear();
    arena1.clear();
    arena1.clear();

    println!("\nAfter clearing arena1 three times:");
    println!("  Arena1 generation: {}", arena1.generation());
    println!("  Arena2 generation: {}", arena2.generation());
    println!("  ID2 still has generation: {}", id2.generation());

    // Now create a new handle in arena1 with the same generation as ID2
    let new_id1 = arena1.alloc("New data").unwrap();

    // Advance arena1 to match ID2's generation
    while arena1.generation() != id2.generation() {
        arena1.clear();
        if arena1.generation() > 10 { break; } // Safety limit
    }
    let matching_gen_id = arena1.alloc("Matching generation").unwrap();

    println!("\nCreated handle with matching generation:");
    println!("  Arena1 generation: {}", arena1.generation());
    println!("  New ID generation: {}", matching_gen_id.generation());
    println!("  ID2 generation: {}", id2.generation());

    // This is the problem: same generation numbers from different arenas!
    if matching_gen_id.generation() == id2.generation() {
        println!("  ⚠️  Same generation numbers from different arenas!");
    }
}

fn demonstrate_false_security() {
    println!("\n2. False Security: Generation Check Passes");
    println!("------------------------------------------");

    let mut arena1: Arena<1024> = Arena::new();
    let mut arena2: Arena<1024> = Arena::new();

    // Create handles with same generation (0)
    let id1 = arena1.alloc(42u32).unwrap();
    let id2 = arena2.alloc(99u32).unwrap();

    println!("Created handles:");
    println!("  Arena1 has: {}", arena1.get(&id1).unwrap());
    println!("  Arena2 has: {}", arena2.get(&id2).unwrap());

    // The dangerous part: cross-arena access
    println!("\nCross-arena access (DANGEROUS!):");

    // This passes the generation check because both are generation 0!
    match arena1.get(&id2) {
        Some(value) => {
            println!("  Arena1.get(id2) = {} ⚠️  (WRONG! This should fail)", value);
            println!("    - Generation check: {} == {} ✓ (passes)",
                     id2.generation(), arena1.generation());
            println!("    - Bounds check: ✓ (passes - wrong arena's bounds!)");
            println!("    - Returns: Wrong arena's data at that offset");
        }
        None => println!("  Arena1.get(id2) = None ✓ (correctly rejected)"),
    }

    match arena2.get(&id1) {
        Some(value) => {
            println!("  Arena2.get(id1) = {} ⚠️  (WRONG! This should fail)", value);
            println!("    - Generation check: {} == {} ✓ (passes)",
                     id1.generation(), arena2.generation());
        }
        None => println!("  Arena2.get(id1) = None ✓ (correctly rejected)"),
    }
}

fn demonstrate_real_world_scenario() {
    println!("\n3. Real-World Dangerous Scenario");
    println!("--------------------------------");

    // Simulate a game with player and enemy systems
    let mut player_arena: Arena<1024> = Arena::new();
    let mut enemy_arena: Arena<1024> = Arena::new();

    #[derive(Debug)]
    struct Player { health: u32, score: u32 }

    #[derive(Debug)]
    struct Enemy { damage: u32, ai_type: u32 }

    // Create some game entities
    let player_id = player_arena.alloc(Player { health: 100, score: 1000 }).unwrap();
    let enemy_id = enemy_arena.alloc(Enemy { damage: 25, ai_type: 1 }).unwrap();

    println!("Game entities created:");
    println!("  Player: {:?}", player_arena.get(&player_id).unwrap());
    println!("  Enemy: {:?}", enemy_arena.get(&enemy_id).unwrap());

    // Simulate a bug where enemy_id gets passed to player system
    println!("\nBug: Enemy ID passed to player system:");

    // This compiles and runs! 😱
    if let Some(wrong_data) = player_arena.get(&enemy_id) {
        println!("  Got 'player' data: {:?}", wrong_data);
        println!("  ⚠️  This is actually player arena data at enemy_id's offset!");
        println!("  ⚠️  Could be completely wrong data or cause memory corruption!");
    } else {
        println!("  ✓ Correctly rejected (None)");
    }

    // Show what happens with different generations
    println!("\nAfter some game events (clearing player arena):");
    player_arena.clear(); // Players respawn, generation advances

    if let Some(_) = player_arena.get(&enemy_id) {
        println!("  Still getting wrong data!");
    } else {
        println!("  ✓ Now correctly rejected due to generation mismatch");
        println!("    Player arena generation: {}", player_arena.generation());
        println!("    Enemy ID generation: {}", enemy_id.generation());
    }

    println!("\n🎯 CONCLUSION:");
    println!("   Generational indices protect against TEMPORAL bugs (stale handles)");
    println!("   but NOT against SPATIAL bugs (wrong arena).");
    println!("   You need custom markers for complete type safety!");
}
