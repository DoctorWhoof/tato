use core::array::from_fn;

use crate::*;
pub const SPRITES_PER_LINE: usize = 16;
// const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
// const CLUSTER_COUNT: usize = TILE_LEN / PIXELS_PER_CLUSTER as usize;

#[derive(Debug, Clone, Default)]
pub(crate) struct SpriteLine {
    pub x: u16,
    pub pixels: Cluster<2>,
    pub palette: PaletteID,
}

#[derive(Debug, Clone)]
pub struct Scanline {
    pub sprite_count: u8,
    // Tracks Which slots contain sprites
    pub mask: u8,
    // Tracks which sprite "slots" available to this scanline have been used.
    pub sprite_registry: u16,
    pub sprites: [SpriteLine; SPRITES_PER_LINE],
}

#[derive(Debug)]
pub struct SpriteGenerator {
    sprite_head: u8,
    pub(crate) scanlines: [Scanline; 240],
    // pub(crate) generators: [Sprite; SPRITES_PER_LINE],
}

impl SpriteGenerator {
    pub fn new() -> Self {
        Self {
            sprite_head: 0,
            scanlines: from_fn(|_| Scanline {
                mask: 0,
                sprite_count: 0,
                sprite_registry: 0,
                sprites: Default::default(),
            }),
        }
    }

    pub fn reset(&mut self) {
        self.sprite_head = 0;
        for line in &mut self.scanlines {
            line.mask = 0;
            line.sprite_count = 0;
            line.sprite_registry = 0;
        }
    }

    pub fn insert(
        &mut self,
        x: u16,
        y: u16,
        flags: TileFlags,
        tile: &[Cluster<2>],
        screen_width: u16,
        screen_height: u16,
    ) {
        if self.sprite_head as usize == SPRITES_PER_LINE {
            return;
        }
        let width = TILE_SIZE as u16;
        let height = TILE_SIZE as u16;

        // Copy transformed tile data to scanline buffers
        for local_y in 0..height {
            for local_x in 0..width {
                let screen_x = x + local_x;
                if screen_x >= screen_width {
                    continue;
                }

                let screen_y = y + local_y;
                if screen_y >= screen_height {
                    break;
                }

                // Acquire scanline
                let line = &mut self.scanlines[(y + local_y) as usize];
                if line.sprite_count as usize >= SPRITES_PER_LINE {
                    return;
                }

                // Copy source pixel
                let (tx, ty) = transform_tile_coords(local_x, local_y, width, height, flags);
                let source_index = ((ty * width) + tx) as usize;
                let source_cluster = source_index / PIXELS_PER_CLUSTER as usize;
                let source_subpixel = source_index % PIXELS_PER_CLUSTER as usize;
                let source_pixel = tile[source_cluster].get_subpixel(source_subpixel as u8);

                // Write to destination pixel
                let dest_subpixel = local_x as usize; // max tile size is same as cluster size
                let sprite_index = line.sprite_count as usize;
                let sprite = &mut line.sprites[sprite_index];
                sprite
                    .pixels
                    .set_subpixel(source_pixel, dest_subpixel as u8);

                // Bit mask
                let slot = ((screen_x as f32 / screen_width as f32) * 8.0).floor() as usize;
                // Ensure slot is in valid range
                debug_assert!(slot < 8, err!("Invalid slot index"));
                // "tell" the scanline which slot we're using
                line.mask |= 1 << slot;

                // Registry
                let sprite_bit = 1u16 << line.sprite_count;
                if line.sprite_registry & sprite_bit == 0 {
                    // Sprite hasn't been registered yet, register it
                    line.sprite_registry |= sprite_bit;
                    sprite.palette = flags.palette();
                    sprite.x = x;
                    line.sprite_count += 1;
                }
            }
        }

        self.sprite_head += 1;
    }
}

#[inline(always)]
fn transform_tile_coords(x: u16, y: u16, w: u16, h: u16, flags: TileFlags) -> (u16, u16) {
    // Handle both rotation and flipping
    if flags.is_rotated() {
        // For 90° clockwise rotation, swap x and y and flip the new x axis
        let rotated_x = h - 1 - y;
        let rotated_y = x;

        // Apply additional flipping if needed
        if flags.is_flipped_x() {
            // Flipping X after 90° rotation is equivalent to flipping the new Y
            (rotated_x, w - 1 - rotated_y)
        } else if flags.is_flipped_y() {
            // Flipping Y after 90° rotation is equivalent to flipping the new X
            (h - 1 - rotated_x, rotated_y)
        } else {
            (rotated_x, rotated_y)
        }
    } else {
        // Handle just flipping without rotation
        let flipped_x = if flags.is_flipped_x() { w - 1 - x } else { x };
        let flipped_y = if flags.is_flipped_y() { h - 1 - y } else { y };
        (flipped_x, flipped_y)
    }
}
