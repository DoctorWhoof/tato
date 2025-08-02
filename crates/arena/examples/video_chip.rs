//! Simple video chip simulation using arena pools
#![allow(unused)]

use tato_arena::Arena;

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, Copy, Default)]
struct Sprite {
    x: u8,
    y: u8,
    tile: u8,
    palette: u8,
}

struct VideoChip {
    arena: Arena<4096>,
    palette: Option<tato_arena::Pool<Color>>,
    sprites: Option<tato_arena::Pool<Sprite>>,
}

impl VideoChip {
    fn new() -> Self {
        Self {
            arena: Arena::new(),
            palette: None,
            sprites: None,
        }
    }

    fn init_palette(&mut self) {
        // Create a palette of 16 colors
        let palette = self.arena.alloc_pool_from_fn(16, |i| {
            Color {
                r: (i * 16) as u8,
                g: ((i * 32) % 256) as u8,
                b: ((i * 64) % 256) as u8,
            }
        }).unwrap();

        self.palette = Some(palette);
    }

    fn init_sprites(&mut self, count: usize) {
        let sprites = self.arena.alloc_pool::<Sprite>(count).unwrap();
        self.sprites = Some(sprites);
    }

    fn set_sprite(&mut self, index: usize, sprite: Sprite) {
        if let Some(ref sprites) = self.sprites {
            if let Some(sprite_slice) = self.arena.get_pool_mut(sprites) {
                if index < sprite_slice.len() {
                    sprite_slice[index] = sprite;
                }
            }
        }
    }

    fn get_color(&self, index: usize) -> Option<Color> {
        let palette = self.palette.as_ref()?;
        let colors = self.arena.get_pool(palette)?;
        colors.get(index).copied()
    }

    fn render_line(&self, line: u8) -> Vec<u8> {
        let mut buffer = vec![0u8; 256];

        if let Some(ref sprites) = self.sprites {
            if let Some(sprite_slice) = self.arena.get_pool(sprites) {
                for sprite in sprite_slice {
                    if sprite.y <= line && line < sprite.y + 8 {
                        if sprite.x < 255 {
                            buffer[sprite.x as usize] = sprite.palette;
                        }
                    }
                }
            }
        }

        buffer
    }
}

fn main() {
    println!("=== Video Chip Demo ===");

    let mut chip = VideoChip::new();

    // Initialize palette
    chip.init_palette();
    println!("Initialized 16-color palette");

    // Show some palette colors
    for i in 0..4 {
        if let Some(color) = chip.get_color(i) {
            println!("Color {}: RGB({}, {}, {})", i, color.r, color.g, color.b);
        }
    }

    // Initialize sprites
    chip.init_sprites(64);
    println!("Initialized 64 sprites");

    // Set some sprites
    chip.set_sprite(0, Sprite { x: 10, y: 20, tile: 1, palette: 5 });
    chip.set_sprite(1, Sprite { x: 50, y: 20, tile: 2, palette: 3 });
    chip.set_sprite(2, Sprite { x: 100, y: 25, tile: 3, palette: 7 });

    // Render a scanline
    let line_data = chip.render_line(22);
    println!("Rendered scanline 22");

    // Show non-zero pixels
    for (x, &pixel) in line_data.iter().enumerate() {
        if pixel != 0 {
            println!("Pixel at x={}: palette={}", x, pixel);
        }
    }

    println!("Arena used: {} bytes", chip.arena.used());
}
