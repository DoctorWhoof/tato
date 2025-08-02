//! This example demonstrates the key safety features:
//! - Generational safety (prevents use-after-restore)
//! - Cross-arena safety (prevents mixing handles between arenas)
//! - Automatic validation

use tato_arena::Arena;

#[derive(Debug)]
struct GameData {
    score: u32,
    level: u8,
}

fn main() {
    println!("=== Arena Safety Features ===\n");
    generational_safety_demo();
    cross_arena_safety_demo();
    restore_safety_demo();
}

fn generational_safety_demo() {
    println!("1. Generational Safety");
    println!("---------------------");

    let mut arena: Arena<1024> = Arena::new();

    // Allocate some data
    let data_id = arena.alloc(GameData { score: 1000, level: 5 }).unwrap();
    let data = arena.get(&data_id).unwrap();
    println!("Initial data: {:?}", data);
    println!("  Score: {}, Level: {}", data.score, data.level);
    println!("Arena generation: {}", arena.generation());

    // Clear the arena - this increments the generation
    arena.clear();
    println!("After clear - Arena generation: {}", arena.generation());

    // The old handle is now safely invalid
    match arena.get(&data_id) {
        Some(data) => println!("Old data: {:?}", data),
        None => println!("✓ Old handle safely rejected (generation mismatch)"),
    }

    // New allocations work fine
    let new_id = arena.alloc(GameData { score: 2000, level: 1 }).unwrap();
    let new_data = arena.get(&new_id).unwrap();
    println!("New data: {:?}", new_data);
    println!("  Score: {}, Level: {}", new_data.score, new_data.level);
    println!();
}

fn cross_arena_safety_demo() {
    println!("2. Cross-Arena Safety");
    println!("--------------------");

    // Create two separate arenas
    let mut arena1: Arena<1024> = Arena::new();
    let mut arena2: Arena<1024> = Arena::new();

    println!("Arena1 ID: 0x{:08X}", arena1.arena_id());
    println!("Arena2 ID: 0x{:08X}", arena2.arena_id());

    // Allocate data in each arena
    let data1 = arena1.alloc(GameData { score: 100, level: 1 }).unwrap();
    let data2 = arena2.alloc(GameData { score: 200, level: 2 }).unwrap();

    // Correct usage works
    println!("Arena1 data: {:?}", arena1.get(&data1).unwrap());
    println!("Arena2 data: {:?}", arena2.get(&data2).unwrap());

    // Cross-arena access is automatically blocked
    match arena1.get(&data2) {
        Some(_) => println!("❌ Cross-arena access worked (shouldn't happen!)"),
        None => println!("✓ Cross-arena access blocked (arena ID mismatch)"),
    }

    match arena2.get(&data1) {
        Some(_) => println!("❌ Cross-arena access worked (shouldn't happen!)"),
        None => println!("✓ Cross-arena access blocked (arena ID mismatch)"),
    }
    println!();
}

fn restore_safety_demo() {
    println!("3. Restore-to Safety");
    println!("-------------------");

    let mut arena: Arena<1024> = Arena::new();

    // Phase 1: Initial allocation
    let initial_data = arena.alloc(GameData { score: 500, level: 3 }).unwrap();
    let checkpoint = arena.used();
    println!("Checkpoint saved at {} bytes", checkpoint);

    // Phase 2: More allocations
    let temp_data1 = arena.alloc(GameData { score: 600, level: 4 }).unwrap();
    let temp_data2 = arena.alloc(GameData { score: 700, level: 5 }).unwrap();

    println!("Before restore:");
    println!("  Initial: {:?}", arena.get(&initial_data).unwrap());
    println!("  Temp1: {:?}", arena.get(&temp_data1).unwrap());
    println!("  Temp2: {:?}", arena.get(&temp_data2).unwrap());

    // Restore to checkpoint - this invalidates ALL handles!
    arena.restore_to(checkpoint);
    println!("Restored to checkpoint");

    // ALL handles are now invalid due to generation increment
    println!("After restore:");
    match arena.get(&initial_data) {
        Some(_) => println!("  Initial: Still valid"),
        None => println!("  Initial: ✓ Invalidated (generation changed)"),
    }

    match arena.get(&temp_data1) {
        Some(_) => println!("  Temp1: Still valid"),
        None => println!("  Temp1: ✓ Invalidated (generation changed)"),
    }

    match arena.get(&temp_data2) {
        Some(_) => println!("  Temp2: Still valid"),
        None => println!("  Temp2: ✓ Invalidated (generation changed)"),
    }

    // Can allocate new data in the restored space
    let new_data = arena.alloc(GameData { score: 999, level: 10 }).unwrap();
    println!("  New data: {:?}", arena.get(&new_data).unwrap());

    println!("\n🛡️  All safety features work automatically - no unsafe behavior possible!");
}
