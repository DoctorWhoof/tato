# Arena Examples

This directory contains simple examples demonstrating the key features of the `tato_arena` crate.

## Examples

### `generational_safety.rs`
Demonstrates the generational safety system that prevents use-after-restore bugs:
- Basic generational safety with `clear()`
- Safe `restore_to()` operations
- Type marker safety between different arenas
- Pool generational safety

```bash
cargo run --example generational_safety
```

### `pool_demo.rs`
Shows basic pool allocation for runtime-sized arrays:
- Creating pools with `alloc_pool()`
- Initializing pools with functions using `alloc_pool_from_fn()`
- Accessing pool data safely

```bash
cargo run --example pool_demo
```

### `video_chip.rs`
Simulates a simple video chip using arena pools:
- Color palette management
- Sprite management with pools
- Scanline rendering simulation

```bash
cargo run --example video_chip
```

### `generic_storage.rs`
Demonstrates type marker safety and generic storage:
- Using type markers to prevent cross-arena access
- Storing different generic types safely
- Raw ID conversion for serialization

```bash
cargo run --example generic_storage
```

### `smaller_indices.rs`
Shows memory savings from using smaller index types:
- Comparing `usize`, `u16`, and `u8` index sizes
- Practical capacity limits for different index types
- Memory usage optimization

```bash
cargo run --example smaller_indices
```

## Key Features Demonstrated

- **Generational Safety**: Handles become invalid after `restore_to()` or `clear()`
- **Type Markers**: Compile-time prevention of cross-arena handle usage
- **Pool Allocation**: Runtime-sized arrays with lightweight handles
- **Custom Index Types**: Use `u8`, `u16` instead of `usize` to save memory (via `ArenaIndex` trait)
- **No-std Support**: All examples work without the standard library
- **Clean API**: `ArenaIndex` trait consolidates type constraints for readable signatures

## API Changes from v0.1.0

The main change is that access methods now return `Option<>` for safety:

```rust
// Old API
let value = arena.get(&id);

// New API  
let value = arena.get(&id)?;  // or .unwrap()
```

This ensures stale handles are caught at runtime rather than causing undefined behavior.

## ArenaIndex Trait

The `ArenaIndex` trait consolidates all requirements for index types, making function signatures much cleaner:

```rust
// Before: Verbose type constraints
where SizeType: Copy + TryFrom<usize> + Into<usize> + PartialOrd + core::ops::Add<Output = SizeType>

// After: Clean and simple
where SizeType: ArenaIndex
```

Supported index types:
- `u8`: 0-255 bytes (8-byte handles)
- `u16`: 0-65,535 bytes (8-byte handles) 
- `usize`: Full system capacity (24-byte handles on 64-bit)