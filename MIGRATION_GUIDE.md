# Migration Guide: Backend-Agnostic Debug UI

This guide shows how to migrate from the old backend-embedded debug UI to the new backend-agnostic `DebugRenderer` system.

## What Changed

### ✅ **Before (Old System)**
- Debug UI was built into `RaylibBackend.render()` 
- Debug input handled inside backend
- Debug features tied to raylib
- Hard to extend with game-specific debug info

### 🎯 **After (New System)**  
- `DebugRenderer` generates backend-agnostic drawing commands
- Games handle debug input and call debug methods
- Same debug UI works with any backend (raylib, SDL, wgpu, etc.)
- Easy to add entity inspection, collision debug, custom overlays

## Migration Steps

### Step 1: Add DebugRenderer to your game

```rust
// OLD: Just backend
let mut backend = RaylibBackend::new(&tato);

// NEW: Add debug renderer
let mut backend = RaylibBackend::new(&tato);
let mut debug_renderer = DebugRenderer::new();
debug_renderer.enabled = true; // or false to start disabled
```

### Step 2: Handle debug input in your game loop

```rust
// OLD: Backend handled this automatically
// (no code needed)

// NEW: Handle in game loop
let tab_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_TAB);
let plus_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_EQUAL);
let minus_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_MINUS);
debug_renderer.handle_standard_input(tab_pressed, plus_pressed, minus_pressed);
```

### Step 3: Call debug rendering

```rust
// OLD: Debug UI was part of backend.render()
backend.render(&mut tato, &[&bg_map]);

// NEW: Separate main rendering from debug rendering
backend.render(&tato, &[&bg_map]);  // Note: &tato instead of &mut tato
debug_renderer.render_and_execute(&mut backend, &tato);
```

## Complete Example Migration

### Before (examples/tilemap/src/main.rs)

```rust
use tato::prelude::*;
use tato_raylib::*;

// ... setup code ...

let mut backend = RaylibBackend::new(&tato);
backend.bg_color = raylib::prelude::Color::BLACK;

while !backend.ray.window_should_close() {
    tato.frame_start(backend.ray.get_frame_time());
    backend.update_gamepad(&mut tato.pad);

    // game logic...
    
    tato.frame_finish();
    backend.render(&mut tato, &[&bg_map]);  // Had debug UI built-in
}
```

### After (with new DebugRenderer)

```rust
use tato::prelude::*;
use tato_raylib::*;

// ... setup code ...

let mut backend = RaylibBackend::new(&tato);
backend.bg_color = raylib::prelude::Color::BLACK;

// NEW: Add debug renderer
let mut debug_renderer = DebugRenderer::new();
debug_renderer.enabled = true;

while !backend.should_close() {  // NEW: use trait method
    tato.frame_start(backend.ray.get_frame_time());
    backend.update_input(&mut tato.pad);  // NEW: use trait method
    
    // NEW: Handle debug input
    let tab_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_TAB);
    let plus_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_EQUAL);
    let minus_pressed = backend.ray.is_key_pressed(KeyboardKey::KEY_MINUS);
    debug_renderer.handle_standard_input(tab_pressed, plus_pressed, minus_pressed);

    // game logic (unchanged)...
    
    tato.frame_finish();
    
    // NEW: Separate rendering
    backend.render(&tato, &[&bg_map]);  // Core rendering only
    debug_renderer.render_and_execute(&mut backend, &tato);  // Debug UI
}
```

## New Capabilities

### Game-Specific Debug Features

Now you can easily add custom debug visualizations:

```rust
if debug_renderer.enabled {
    // Entity inspection
    debug_renderer.debug_entity("Player", player.x, player.y, &[
        ("Health", player.health.to_string()),
        ("State", format!("{:?}", player.state)),
        ("Velocity", format!("{:.1}, {:.1}", player.vx, player.vy)),
    ]);
    
    // Collision visualization
    debug_renderer.debug_collision_box(player.collision_rect, None);
    
    // Custom drawing commands
    debug_renderer.add_command(DrawOp::Text {
        text: format!("Entities: {}", entities.len()),
        x: 10.0,
        y: 250.0,
        size: 12.0,
        color: RGBA32 { r: 255, g: 255, b: 0, a: 255 },
    });
}
```

### Backend Independence

The same debug code now works with any backend:

```rust
// Works with raylib
let mut raylib_backend = RaylibBackend::new(&tato);
debug_renderer.render_and_execute(&mut raylib_backend, &tato);

// Will work with SDL, wgpu, etc. when available
// let mut sdl_backend = SdlBackend::new(&tato);
// debug_renderer.render_and_execute(&mut sdl_backend, &tato);
```

## Benefits Summary

✅ **Same Visual Result**: Debug UI looks identical to the old system  
✅ **Backend Agnostic**: Works with raylib, SDL, wgpu, etc.  
✅ **Game-Specific Debug**: Easy entity inspection, collision visualization  
✅ **Extensible**: Add custom debug overlays and tools  
✅ **Cleaner Architecture**: Backend focuses on rendering, game controls debug features  
✅ **Easy Migration**: Minimal code changes required  

## Troubleshooting

### "Method not found" errors
Make sure you're importing the debug module:
```rust
use tato::prelude::*;  // This includes DebugRenderer
```

### Debug UI doesn't appear
Check that debug is enabled:
```rust
debug_renderer.enabled = true;
```

### Performance concerns
The new system generates the same drawing commands as before, so performance should be identical or better.

## Next Steps

- Try adding entity debug visualization to your game
- Experiment with custom debug overlays using `debug_renderer.add_command()`
- Consider creating game-specific debug systems that use DebugRenderer internally

## DrawOp Commands

The debug system uses `DrawOp` enum for backend-agnostic drawing commands:

```rust
pub enum DrawOp {
    Rect { x: i16, y: i16, w: i16, h: i16, color: RGBA32 },
    Text { text: String, x: f32, y: f32, size: f32, color: RGBA32 },
    Line { x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32 },
    Texture { id: TextureId, x: f32, y: f32, scale: f32, tint: RGBA32 },
}
```

Add custom drawing commands like this:
```rust
debug_renderer.add_command(DrawOp::Rect {
    x: 100, y: 100, w: 50, h: 30,
    color: RGBA32 { r: 255, g: 0, b: 0, a: 128 }
});
```