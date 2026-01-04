//! Example demonstrating the internal tail allocation optimization in Text operations.
//! 
//! This shows how Text formatting works efficiently even on microcontrollers
//! with minimal stack space, by using the arena's own memory for temporary buffers.

use tato_arena::{Arena, ArenaOps, Text};

fn main() {
    println!("=== Tail Allocation Demo ===\n");
    
    // Create a small arena - imagine this is on a microcontroller with limited RAM
    let mut arena: Arena<512> = Arena::new();
    println!("Created arena with {} bytes capacity", arena.capacity());
    
    // Traditional approach would need stack buffers:
    // - DebugBuffer uses 256 bytes on stack
    // - from_buffer used 4096 bytes on stack (!)
    // - join used a 1024-byte Arena on stack
    // 
    // With tail allocation, NO stack buffers are needed!
    
    println!("\n1. Text formatting (no 256-byte stack buffer needed!):");
    let initial_used = arena.used();
    let text1 = Text::format(&mut arena, "Temperature: {}", 23.5, "C").unwrap();
    println!("   Formatted: '{}'", text1.as_str(&arena).unwrap());
    println!("   Space used: {} bytes (just the final string)", arena.used() - initial_used);
    
    println!("\n2. Multiple value formatting:");
    let before = arena.used();
    let values = [1, 2, 3, 4, 5];
    let text2 = Text::format_display(
        &mut arena,
        "Sensors: {}, {}, {}, {}, {}",
        &values,
        " OK"
    ).unwrap();
    println!("   Formatted: '{}'", text2.as_str(&arena).unwrap());
    println!("   Space used: {} bytes", arena.used() - before);
    
    println!("\n3. Text joining (no 1KB temp arena on stack!):");
    let t1 = Text::from_str(&mut arena, "System").unwrap();
    let t2 = Text::from_str(&mut arena, " ").unwrap();
    let t3 = Text::from_str(&mut arena, "Ready").unwrap();
    let before = arena.used();
    let joined = Text::join(&mut arena, &[t1, t2, t3]).unwrap();
    println!("   Joined: '{}'", joined.as_str(&arena).unwrap());
    println!("   Space used: {} bytes (just the final string)", arena.used() - before);
    
    println!("\n4. Stress test - many operations:");
    let before = arena.used();
    for i in 0..5 {
        let msg = Text::format(&mut arena, "Log {}: ", i, "OK").unwrap();
        println!("   {}", msg.as_str(&arena).unwrap());
    }
    let after = arena.used();
    println!("   Total space for 5 messages: {} bytes", after - before);
    
    println!("\n5. Works with ArenaRef too (size-erased):");
    let mut arena_ref = arena.as_ref();
    let text3 = Text::format(&mut arena_ref, "Via ref: {}", 999, "!").unwrap();
    println!("   Formatted: '{}'", text3.as_str(&arena_ref).unwrap());
    
    println!("\n=== Summary ===");
    println!("Total arena used: {}/{} bytes", arena.used(), arena.capacity());
    println!("Remaining: {} bytes", arena.remaining());
    
    println!("\n✅ Benefits of tail allocation:");
    println!("   - No stack buffers needed (safe for embedded)");
    println!("   - Works with tiny arenas (128-512 bytes)");
    println!("   - Temporary space automatically reclaimed");
    println!("   - Scales with arena size, not stack size");
    println!("   - Zero overhead - compiles to efficient code");
    
    println!("\n❌ Without tail allocation, we'd need:");
    println!("   - 256 bytes for DebugBuffer");
    println!("   - 4096 bytes for from_buffer temp");
    println!("   - 1024 bytes for join temp arena");
    println!("   = 5376 bytes of stack! (More than many MCUs have!)");
}