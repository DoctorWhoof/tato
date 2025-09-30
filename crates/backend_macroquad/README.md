# Tato Macroquad Backend

A [Macroquad](https://github.com/not-fl3/macroquad)-based rendering backend for the Tato game development framework.

## Overview

This backend provides a Macroquad-based implementation of the Tato `Backend` trait, allowing you to render Tato games using Macroquad's cross-platform graphics capabilities.

## Features

- Cross-platform rendering (Windows, macOS, Linux, Web, Android, iOS)
- Hardware-accelerated graphics via Macroquad
- Font rendering support
- Texture management
- Input handling (keyboard, mouse)
- Configurable rendering options

## Usage

Add the backend to your `Cargo.toml`:

```toml
[dependencies]
tato = { path = "path/to/tato" }
tato_macroquad = { path = "path/to/tato/crates/backend_macroquad" }
macroquad = "0.4"
```

### Basic Example

```rust
use macroquad::prelude::*;
use tato::{arena::Arena, prelude::*};
use tato_macroquad::*;

// Configure window with VSync and proper settings
fn window_conf() -> Conf {
    tato_window_conf("My Tato Game", 900, 540)
}

#[macroquad::main(window_conf)]
async fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<32_768, u32>::new();
    let mut bg_map = Tilemap::<896>::new(32, 28);
    let mut tato = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new().unwrap();

    // Initialize the backend
    let mut backend = MquadBackend::new(&tato).await;
    
    // Main game loop
    loop {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);
        
        // Handle window close
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Your game logic here...
        
        // Render
        tato.frame_finish();
        backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
        
        next_frame().await;
    }
    
    Ok(())
}
```

## Key Differences from Raylib Backend

### Window Configuration with VSync
Macroquad requires window configuration before the main function runs:

```rust
// Configure window with VSync enabled
fn window_conf() -> Conf {
    tato_window_conf("My Game", 900, 540)
}

#[macroquad::main(window_conf)]
async fn main() -> TatoResult<()> {
    // Game code here
}
```

For custom configurations, you can also create your own `Conf`:

```rust
fn window_conf() -> Conf {
    Conf {
        window_title: "My Game".to_owned(),
        window_width: 900,
        window_height: 540,
        window_resizable: true,
        high_dpi: true,
        // VSync is enabled by default in Macroquad
        ..Default::default()
    }
}
```

### Async Initialization
The Macroquad backend requires async initialization:

```rust
// Raylib (sync)
let mut backend = RayBackend::new(&tato);

// Macroquad (async)
let mut backend = MquadBackend::new(&tato).await;
```

### Main Loop Structure
Macroquad requires a different main loop structure:

```rust
// Raylib
while !backend.should_close() {
    // game logic
    backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
}

// Macroquad
loop {
    // Handle close manually
    if is_key_pressed(KeyCode::Escape) {
        break;
    }
    
    // game logic
    backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
    
    next_frame().await; // Required!
}
```

### Window Management
- Window title setting is not supported at runtime in Macroquad
- Window close detection works differently
- FPS control is handled by Macroquad's main loop

## Input Mapping

The backend maps keyboard inputs to Tato's button system:

| Tato Button | Keyboard Key |
|-------------|--------------|
| Left        | Left Arrow   |
| Right       | Right Arrow  |
| Up          | Up Arrow     |
| Down        | Down Arrow   |
| A           | Z            |
| B           | X            |
| X           | A            |
| Y           | S            |
| Menu        | Escape       |
| Start       | Enter        |
| L Shoulder  | Q            |
| R Shoulder  | W            |

## VSync and Performance

The Macroquad backend automatically enables VSync through the window configuration. Key performance features:

- **VSync**: Enabled by default to prevent screen tearing
- **Texture Filtering**: Disabled (`FilterMode::Nearest`) for crisp pixel art
- **High DPI**: Supported for sharp rendering on retina displays
- **Hardware Acceleration**: OpenGL/WebGL rendering for smooth performance

Use the `tato_window_conf()` helper function for optimal settings:

```rust
fn window_conf() -> Conf {
    tato_window_conf("My Game", 900, 540)
}
```

## Backend Structure

The `MquadBackend` struct contains:

- `bg_color`: Background clear color
- `integer_scaling`: Whether to use integer scaling for pixel-perfect rendering
- `print_frame_time`: Debug option to print frame timing information
- `canvas_rect`: Optional canvas rendering rectangle
- Internal texture management and rendering state

## Running Examples

To run the included example:

```bash
cd tato/crates/backend_macroquad
cargo run --example simple
```

## Platform Support

This backend supports all platforms that Macroquad supports:

- **Desktop**: Windows, macOS, Linux
- **Web**: WebAssembly via wasm-pack
- **Mobile**: Android, iOS (with additional setup)

For web deployment, use:

```bash
cargo build --target wasm32-unknown-unknown --example simple
```

## Contributing

This backend aims to maintain API compatibility with the Raylib backend while leveraging Macroquad's strengths. When contributing:

1. Keep the `Backend` trait implementation consistent
2. Maintain similar performance characteristics
3. Document any platform-specific behavior
4. Test on multiple platforms when possible

## License

Same as the main Tato project.