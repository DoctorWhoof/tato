//! Video Chip Emulator Example
//!
//! This example demonstrates how to use the arena for a retro video chip
//! emulator where different configurations need different memory layouts.

use tato_arena::{Arena, ArenaId, Pool};

/// A simple RGB color (3 bytes)
#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

/// A sprite definition (8 bytes)
#[derive(Debug, Clone, Copy, Default)]
struct Sprite {
    x: u8,
    y: u8,
    tile_id: u8,
    palette: u8,
    flags: u8,       // flip, priority, etc.
    _padding: [u8; 3], // Align to 8 bytes
}

/// An 8x8 tile (64 bytes of pixel data)
#[derive(Debug, Clone, Copy)]
struct Tile {
    pixels: [u8; 64], // 8x8 pixels, each pixel is palette index
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            pixels: [0; 64],
        }
    }
}

/// Video chip configuration
#[derive(Debug, Clone, Copy)]
struct VideoConfig {
    screen_width: u16,
    screen_height: u16,
    max_sprites: u8,
    max_colors: u8,
    tile_count: u16,
}

/// Our video chip with 16KB of memory
pub struct VideoChip {
    arena: Arena<16384>, // 16KB total

    // Configuration
    config: ArenaId<VideoConfig>,

    // Video data
    palette: Option<ArenaId<[Color; 256]>>,
    sprites: Option<Pool<Sprite>>, // Runtime-sized sprite collection
    tiles: Option<Pool<Tile>>,     // Runtime-sized tile collection
    scanline_buffer: Option<ArenaId<[u8; 256]>>, // One scanline buffer
}



impl VideoChip {
    /// Create a new video chip with the given configuration
    pub fn new(config: VideoConfig) -> Option<Self> {
        let mut arena = Arena::new();

        // Store the configuration
        let config_id = arena.alloc(config)?;

        let mut chip = Self {
            arena,
            config: config_id,
            palette: None,
            sprites: None,
            tiles: None,
            scanline_buffer: None,
        };

        // Initialize based on config
        chip.init_components()?;

        Some(chip)
    }

    fn init_components(&mut self) -> Option<()> {
        let config = *self.arena.get(&self.config);
        
        // Allocate color palette
        let palette = [Color { r: 0, g: 0, b: 0 }; 256];
        self.palette = Some(self.arena.alloc(palette)?);
        
        // Allocate sprite and tile collections based on config
        self.sprites = Some(self.arena.alloc_pool::<Sprite>(config.max_sprites as usize)?);
        self.tiles = Some(self.arena.alloc_pool::<Tile>(config.tile_count as usize)?);
        
        // Allocate scanline buffer
        let scanline = [0u8; 256];
        self.scanline_buffer = Some(self.arena.alloc(scanline)?);
        
        Some(())
    }

    /// Set a color in the palette
    pub fn set_color(&mut self, index: u8, color: Color) {
        if let Some(palette_id) = &self.palette {
            let palette = self.arena.get_mut(palette_id);
            palette[index as usize] = color;
        }
    }

    /// Add a sprite to the chip (returns index in sprite collection)
    pub fn add_sprite(&mut self, sprite: Sprite) -> Result<usize, &'static str> {
        if let Some(sprites_vec) = &self.sprites {
            let sprites = self.arena.get_pool_mut(sprites_vec);
            
            // Find first empty slot (sprite with x=0, y=0 is considered empty)
            for (i, existing_sprite) in sprites.iter_mut().enumerate() {
                if existing_sprite.x == 0 && existing_sprite.y == 0 {
                    *existing_sprite = sprite;
                    return Ok(i);
                }
            }
            
            Err("No free sprite slots")
        } else {
            Err("Sprites not initialized")
        }
    }

    /// Add a tile to the chip (returns index in tile collection)
    pub fn add_tile(&mut self, tile: Tile, index: usize) -> Result<(), &'static str> {
        if let Some(tiles_vec) = &self.tiles {
            let tiles = self.arena.get_pool_mut(tiles_vec);
            
            if index < tiles.len() {
                tiles[index] = tile;
                Ok(())
            } else {
                Err("Tile index out of bounds")
            }
        } else {
            Err("Tiles not initialized")
        }
    }

    /// Get a sprite by index
    pub fn get_sprite(&self, index: usize) -> Option<&Sprite> {
        if let Some(sprites_vec) = &self.sprites {
            let sprites = self.arena.get_pool(sprites_vec);
            sprites.get(index)
        } else {
            None
        }
    }
    
    /// Get a mutable sprite by index
    pub fn get_sprite_mut(&mut self, index: usize) -> Option<&mut Sprite> {
        if let Some(sprites_vec) = &self.sprites {
            let sprites = self.arena.get_pool_mut(sprites_vec);
            sprites.get_mut(index)
        } else {
            None
        }
    }

    /// Simulate rendering a scanline
    pub fn render_scanline(&mut self, line: u8) {
        if let (Some(buffer_id), Some(sprites_vec)) = (&self.scanline_buffer, &self.sprites) {
            // Collect sprite data first to avoid borrowing conflicts
            let mut sprites_to_render = [Sprite { x: 0, y: 0, tile_id: 0, palette: 0, flags: 0, _padding: [0; 3] }; 128];
            let mut sprite_count = 0;
            
            let sprites = self.arena.get_pool(sprites_vec);
            for sprite in sprites.iter() {
                // Check if sprite is on this scanline (and not empty)
                if sprite.y <= line && line < sprite.y + 8 && sprite_count < 128 && 
                   !(sprite.x == 0 && sprite.y == 0) {
                    sprites_to_render[sprite_count] = *sprite;
                    sprite_count += 1;
                }
            }
            
            // Now render to buffer
            let buffer = self.arena.get_mut(buffer_id);
            
            // Clear the scanline
            buffer.fill(0);
            
            // Render sprites on this scanline
            for i in 0..sprite_count {
                let sprite = sprites_to_render[i];
                let _sprite_line = line - sprite.y;
                
                // In a real implementation, you'd:
                // 1. Get the tile data
                // 2. Get the correct line of pixels
                // 3. Apply palette
                // 4. Handle transparency
                // 5. Composite onto scanline buffer
                
                // For demo, just mark the sprite's X position
                if (sprite.x as usize) < buffer.len() {
                    buffer[sprite.x as usize] = sprite.palette;
                }
            }
        }
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> (usize, usize, usize) {
        let used = self.arena.used();
        let total = 16384;
        let allocations = self.arena.allocation_count();

        (used, total, allocations)
    }

    /// Reset the chip (for switching games/modes)
    pub fn reset(&mut self) {
        self.arena.clear();
        self.palette = None;
        self.sprites = None;
        self.tiles = None;
        self.scanline_buffer = None;
    }
}

fn main() {
    // Create a video chip configuration like a Game Boy
    let config = VideoConfig {
        screen_width: 160,
        screen_height: 144,
        max_sprites: 40,
        max_colors: 4,
        tile_count: 64,  // Reduced for demo - 64 tiles * 64 bytes = 4KB
    };

    let mut chip = VideoChip::new(config).expect("Failed to create video chip");

    // Set up a simple 4-color palette
    chip.set_color(0, Color { r: 155, g: 188, b: 15 });  // Light green
    chip.set_color(1, Color { r: 139, g: 172, b: 15 });  // Medium green
    chip.set_color(2, Color { r: 48, g: 98, b: 48 });    // Dark green
    chip.set_color(3, Color { r: 15, g: 56, b: 15 });    // Darkest green

    // Add some sprites
    let sprite1 = Sprite {
        x: 32,
        y: 64,
        tile_id: 1,
        palette: 1,
        flags: 0,
        _padding: [0; 3],
    };

    let sprite2 = Sprite {
        x: 48,
        y: 64,
        tile_id: 2,
        palette: 2,
        flags: 0,
        _padding: [0; 3],
    };

    let sprite1_idx = chip.add_sprite(sprite1).expect("Failed to add sprite");
    let _sprite2_idx = chip.add_sprite(sprite2).expect("Failed to add sprite");

    // Add a tile
    let mut tile = Tile {
        pixels: [0; 64],
    };

    // Create a simple 8x8 pattern (diagonal)
    for i in 0..8 {
        tile.pixels[i * 8 + i] = 1;
    }

    chip.add_tile(tile, 0).expect("Failed to add tile");

    // Modify sprite1 position
    if let Some(sprite1_mut) = chip.get_sprite_mut(sprite1_idx) {
        sprite1_mut.x = 40;
    }

    // Render a few scanlines
    for line in 60..70 {
        chip.render_scanline(line);
    }

    // Print memory usage
    let (used, total, allocations) = chip.memory_stats();
    println!("Memory usage: {}/{} bytes ({:.1}%)",
             used, total, (used as f32 / total as f32) * 100.0);
    println!("Allocations: {}", allocations);

    // This is where the flexibility shines!
    // We can create different chip configurations without changing the core code:

    // High-end configuration (like a 16-bit console)
    let _high_end_config = VideoConfig {
        screen_width: 320,
        screen_height: 240,
        max_sprites: 128,
        max_colors: 255,
        tile_count: 1024,
    };

    // Low-end configuration (like an 8-bit computer)
    let _low_end_config = VideoConfig {
        screen_width: 128,
        screen_height: 128,
        max_sprites: 16,
        max_colors: 16,
        tile_count: 128,
    };

    println!("Video chip example completed successfully!");
}
