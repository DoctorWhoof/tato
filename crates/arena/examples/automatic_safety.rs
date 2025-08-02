//! Example demonstrating automatic cross-arena safety with random arena IDs

use tato_arena::Arena;

fn main() {
    println!("=== Automatic Cross-Arena Safety Demo ===\n");
    
    demonstrate_automatic_safety();
    demonstrate_custom_rng();
    demonstrate_collision_resistance();
}

fn demonstrate_automatic_safety() {
    println!("1. Automatic Cross-Arena Safety");
    println!("-------------------------------");
    
    // Each arena gets a unique random ID automatically
    let mut arena1: Arena<1024> = Arena::new();
    let mut arena2: Arena<1024> = Arena::new();
    
    println!("Arena IDs (automatically generated):");
    println!("  Arena1 ID: 0x{:08X}", arena1.arena_id());
    println!("  Arena2 ID: 0x{:08X}", arena2.arena_id());
    
    // Create handles
    let id1 = arena1.alloc("Arena1 data").unwrap();
    let id2 = arena2.alloc("Arena2 data").unwrap();
    
    println!("\nHandle arena IDs:");
    println!("  ID1 arena: 0x{:08X}", id1.arena_id());
    println!("  ID2 arena: 0x{:08X}", id2.arena_id());
    
    // ✅ Correct usage works
    println!("\nCorrect usage:");
    println!("  Arena1.get(ID1): {:?}", arena1.get(&id1));
    println!("  Arena2.get(ID2): {:?}", arena2.get(&id2));
    
    // ✅ Cross-arena usage is automatically blocked!
    println!("\nCross-arena usage (automatically blocked):");
    match arena1.get(&id2) {
        Some(data) => println!("  Arena1.get(ID2): {} ⚠️  Should not happen!", data),
        None => println!("  Arena1.get(ID2): None ✓ (arena ID mismatch)"),
    }
    
    match arena2.get(&id1) {
        Some(data) => println!("  Arena2.get(ID1): {} ⚠️  Should not happen!", data),
        None => println!("  Arena2.get(ID1): None ✓ (arena ID mismatch)"),
    }
    
    println!("\n🎯 No custom marker types needed - safety is automatic!");
}

fn demonstrate_custom_rng() {
    println!("\n2. Custom RNG for Reproducible Arena IDs");
    println!("----------------------------------------");
    
    // Use tato_rng for deterministic arena IDs (useful for testing)
    let mut rng = tato_rng::Rng::new(16, 12345);
    
    let mut arena1 = Arena::<1024>::new_with_rng(&mut rng);
    let mut arena2 = Arena::<1024>::new_with_rng(&mut rng);
    let arena3 = Arena::<1024>::new_with_rng(&mut rng);
    
    println!("RNG-generated arena IDs:");
    println!("  Arena1: 0x{:08X}", arena1.arena_id());
    println!("  Arena2: 0x{:08X}", arena2.arena_id());
    println!("  Arena3: 0x{:08X}", arena3.arena_id());
    
    // Test cross-arena safety with predictable IDs
    let id1 = arena1.alloc(100u32).unwrap();
    let id2 = arena2.alloc(200u32).unwrap();
    
    assert!(arena1.get(&id2).is_none());
    assert!(arena2.get(&id1).is_none());
    
    println!("✓ Cross-arena safety verified with tato_rng");
}

fn demonstrate_collision_resistance() {
    println!("\n3. Collision Resistance");
    println!("-----------------------");
    
    // Create many arenas to test for ID collisions
    let mut arena_ids = Vec::new();
    let arena_count = 1000;
    
    println!("Creating {} arenas to test collision resistance...", arena_count);
    
    for _ in 0..arena_count {
        let arena: Arena<1024> = Arena::new();
        arena_ids.push(arena.arena_id());
    }
    
    // Check for collisions
    arena_ids.sort();
    let mut collisions = 0;
    for i in 1..arena_ids.len() {
        if arena_ids[i] == arena_ids[i-1] {
            collisions += 1;
        }
    }
    
    println!("Results:");
    println!("  Total arenas: {}", arena_count);
    println!("  Unique IDs: {}", arena_ids.len() - collisions);
    println!("  Collisions: {}", collisions);
    println!("  Collision rate: {:.4}%", (collisions as f64 / arena_count as f64) * 100.0);
    
    if collisions == 0 {
        println!("✅ Perfect - no collisions detected!");
    } else {
        println!("⚠️  {} collisions detected (extremely rare with 32-bit IDs)", collisions);
    }
    
    // Show some example IDs
    println!("\nSample arena IDs:");
    for i in 0..core::cmp::min(10, arena_ids.len()) {
        println!("  0x{:08X}", arena_ids[i]);
    }
}

#[allow(dead_code)]
fn demonstrate_pools_safety() {
    println!("\n4. Pool Cross-Arena Safety");
    println!("-------------------------");
    
    let mut arena1: Arena<2048> = Arena::new();
    let mut arena2: Arena<2048> = Arena::new();
    
    // Create pools in different arenas
    let pool1 = arena1.alloc_pool_from_fn(5, |i| i as u32 * 10).unwrap();
    let pool2 = arena2.alloc_pool_from_fn(3, |i| i as u32 * 100).unwrap();
    
    println!("Pool arena IDs:");
    println!("  Pool1: 0x{:08X} (from arena1: 0x{:08X})", 
             pool1.arena_id(), arena1.arena_id());
    println!("  Pool2: 0x{:08X} (from arena2: 0x{:08X})", 
             pool2.arena_id(), arena2.arena_id());
    
    // Correct usage
    println!("\nCorrect pool access:");
    if let Some(data) = arena1.get_pool(&pool1) {
        println!("  Arena1.get_pool(pool1): {:?}", data);
    }
    if let Some(data) = arena2.get_pool(&pool2) {
        println!("  Arena2.get_pool(pool2): {:?}", data);
    }
    
    // Cross-arena pool access blocked
    println!("\nCross-arena pool access (blocked):");
    match arena1.get_pool(&pool2) {
        Some(_) => println!("  Arena1.get_pool(pool2): Should not happen!"),
        None => println!("  Arena1.get_pool(pool2): None ✓ (arena ID mismatch)"),
    }
    
    match arena2.get_pool(&pool1) {
        Some(_) => println!("  Arena2.get_pool(pool1): Should not happen!"),
        None => println!("  Arena2.get_pool(pool1): None ✓ (arena ID mismatch)"),
    }
}

// Helper to show arena memory layout
#[allow(dead_code)]
fn show_memory_overhead() {
    println!("\n5. Memory Overhead Analysis");
    println!("---------------------------");
    
    use core::mem::size_of;
    
    println!("Handle sizes with arena ID:");
    println!("  ArenaId<u32, u8>: {} bytes", size_of::<tato_arena::ArenaId<u32, u8>>());
    println!("  ArenaId<u32, u16>: {} bytes", size_of::<tato_arena::ArenaId<u32, u16>>());
    println!("  ArenaId<u32, usize>: {} bytes", size_of::<tato_arena::ArenaId<u32, usize>>());
    
    println!("  Pool<u32, u8>: {} bytes", size_of::<tato_arena::Pool<u32, u8>>());
    println!("  Pool<u32, u16>: {} bytes", size_of::<tato_arena::Pool<u32, u16>>());
    println!("  Pool<u32, usize>: {} bytes", size_of::<tato_arena::Pool<u32, usize>>());
    
    println!("\n💡 Arena ID adds 4 bytes per handle but provides automatic safety!");
}