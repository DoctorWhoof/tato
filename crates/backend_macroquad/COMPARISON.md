# Backend Comparison: Raylib vs Macroquad

This document compares the Tato Raylib and Macroquad backends to help you choose the right one for your project.

## Quick Comparison

| Feature | Raylib Backend | Macroquad Backend |
|---------|---------------|-------------------|
| **Initialization** | Sync (`RayBackend::new()`) | Async (`MquadBackend::new().await`) |
| **Main Loop** | `while !backend.should_close()` | `loop { ... next_frame().await }` |
| **Platform Support** | Desktop (Win/Mac/Linux) | Desktop + Web + Mobile |
| **Window Management** | Full runtime control | Limited runtime control |
| **Performance** | Native C library | Rust + OpenGL/WebGL |
| **Binary Size** | Larger (C dependency) | Smaller (pure Rust) |
| **WebAssembly** | Not supported | Full support |
| **Mobile** | Limited | Full support |

## Detailed Comparison

### Platform Support

#### Raylib Backend
- ✅ Windows (native)
- ✅ macOS (native)
- ✅ Linux (native)
- ❌ Web (WebAssembly)
- ❌ Mobile (Android/iOS)

#### Macroquad Backend
- ✅ Windows (OpenGL)
- ✅ macOS (OpenGL)
- ✅ Linux (OpenGL)
- ✅ Web (WebGL via WASM)
- ✅ Android (OpenGL ES)
- ✅ iOS (Metal/OpenGL ES)

### Code Structure Differences

#### Initialization
```rust
// Raylib - Synchronous
fn main() {
    let backend = RayBackend::new(&tato);
    // Ready to use immediately
}

// Macroquad - Asynchronous
#[macroquad::main("Game Title")]
async fn main() {
    let backend = MquadBackend::new(&tato).await;
    // Must await initialization
}
```

#### Main Game Loop
```rust
// Raylib - Window-managed loop
while !backend.should_close() {
    backend.frame_start(&mut frame_arena, &mut tato.pad);
    // Game logic...
    backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
}

// Macroquad - Manual loop with frame sync
loop {
    if is_key_pressed(KeyCode::Escape) {
        break; // Manual close handling
    }
    
    backend.frame_start(&mut frame_arena, &mut tato.pad);
    // Game logic...
    backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
    
    next_frame().await; // Required for frame sync
}
```

#### Window Management
```rust
// Raylib - Full control
backend.set_window_title("New Title");
backend.set_target_fps(120);
let should_close = backend.should_close();

// Macroquad - Limited runtime control
backend.set_window_title("New Title"); // No-op, set at compile time
backend.set_target_fps(120); // Limited effect, controlled by main loop
let should_close = false; // Always false, handle manually
```

### Performance Characteristics

#### Raylib Backend
- **Pros**:
  - Native C library performance
  - Mature, battle-tested codebase
  - Lower-level control over rendering
  - Direct hardware acceleration
  
- **Cons**:
  - Larger binary size due to C dependencies
  - Platform-specific compilation requirements
  - Less Rust ecosystem integration

#### Macroquad Backend
- **Pros**:
  - Pure Rust implementation
  - Smaller binary sizes
  - Better Rust toolchain integration
  - Cross-compilation friendly
  - WebAssembly ready out-of-the-box
  
- **Cons**:
  - Additional abstraction layer
  - Newer codebase (less mature)
  - OpenGL dependency may limit some platforms

### Use Case Recommendations

#### Choose Raylib Backend When:
- Building desktop-only games
- Need maximum performance
- Require extensive window management features
- Working with existing Raylib ecosystem
- Prefer synchronous initialization
- Binary size is not a concern

#### Choose Macroquad Backend When:
- Targeting multiple platforms (especially web/mobile)
- Want pure Rust dependencies
- Building web games (WebAssembly)
- Prefer smaller binary sizes
- Working in Rust-first environment
- Need easy cross-compilation
- Comfortable with async/await patterns

### Migration Guide

#### From Raylib to Macroquad

1. **Update Cargo.toml**:
```toml
# Replace
tato_raylib = { path = "..." }

# With
tato_macroquad = { path = "..." }
macroquad = "0.4"
```

2. **Update main function**:
```rust
// From
fn main() -> TatoResult<()> {
    let mut backend = RayBackend::new(&tato);
    while !backend.should_close() {
        // game loop
    }
    Ok(())
}

// To
#[macroquad::main("Game Title")]
async fn main() -> TatoResult<()> {
    let mut backend = MquadBackend::new(&tato).await;
    loop {
        if is_key_pressed(KeyCode::Escape) { break; }
        // game loop
        next_frame().await;
    }
    Ok(())
}
```

3. **Update imports**:
```rust
// From
use tato_raylib::*;

// To
use tato_macroquad::*;
use macroquad::prelude::*;
```

4. **Handle window management differences**:
   - Remove runtime `set_window_title()` calls
   - Replace `should_close()` with manual close detection
   - Adjust FPS control expectations

### Conclusion

Both backends provide full compatibility with the Tato `Backend` trait, ensuring your game logic remains unchanged regardless of which backend you choose. The decision primarily depends on your target platforms and development preferences:

- **Raylib**: Best for desktop-focused, performance-critical games
- **Macroquad**: Best for cross-platform games, especially targeting web or mobile

The Macroquad backend's cross-platform capabilities and pure Rust implementation make it an excellent choice for modern game development, while the Raylib backend remains ideal for desktop applications requiring maximum performance and mature tooling.