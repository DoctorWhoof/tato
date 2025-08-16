# API Overview

```rust
// Create arena (u16 index by default, can be higher if needed)
let mut arena: Arena<1024> = Arena::new();

// Allocate values
let id = arena.alloc(42u32)?;
let pool = arena.alloc_pool::<i32>(10)?;

// Access safely (returns Option)
let value = arena.get(&id)?;
let slice = arena.get_pool(&pool)?;

// Memory management
println!("Used: {} bytes", arena.used());
arena.clear(); // Invalidates all handles safely

// For different capacities:
let tiny: Arena<255, u8> = Arena::new();     // 6-byte handles, up to 255 bytes
let default: Arena<65536> = Arena::new();    // 8-byte handles, up to 64KB ⭐
let huge: Arena<1048576, usize> = Arena::new(); // 24-byte handles, unlimited
```

# Arena Examples

### 1. `basic_usage.rs` - **Start with this one!**
The essential introduction to arena usage:
- Single value allocation with `alloc()`
- Safe access with `get()` and `get_mut()`
- Slice allocation with `alloc_pool()` and `alloc_pool_from_fn()`
- Memory management with `clear()`

```bash
cargo run --example basic_usage
```

### 2. `safety_features.rs` - **Why the arena is reliable**
Demonstrates the automatic safety features:
- **Generational safety**: Handles become invalid after `clear()`/`restore_to()`
- **Cross-arena safety**: Can't mix handles between different arenas
- **Restore-to safety**: Checkpoint/restore functionality

```bash
cargo run --example safety_features
```

### 3. `memory_optimization.rs` - **Save memory with smaller indices**
Shows how to use smaller index types for memory efficiency:
- Handle size comparison (`u8`, `u16`, `usize`)
- Capacity limits for different index types
- Practical memory savings calculations

```bash
cargo run --example memory_optimization
```
