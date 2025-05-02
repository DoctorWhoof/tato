use core::array::from_fn;

use crate::*;
const SPRITES_PER_LINE: usize = 16;
const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
const CLUSTER_COUNT: usize = TILE_LEN / PIXELS_PER_CLUSTER as usize;

#[derive(Debug, Clone)]
pub(crate) struct Sprite {
    x: u16,
    w: u16,
    pixels: [Cluster<2>; CLUSTER_COUNT],
    // Stored to provide lazy processing - if they don't change, don't update!
    flags: TileFlags,
    tile_id: TileID,
}

#[derive(Debug, Clone)]
pub struct Scanline {
    pub(crate) mask: u8,
    pub(crate) map: [u16; 8], // each slot can refer to 16 sprite indices (1 bit per sprite)
}

#[derive(Debug)]
pub struct SpriteGenerator {
    sprite_head: u8,
    pub(crate) scanlines: [Scanline; 256],
    pub(crate) sprites: [Sprite; SPRITES_PER_LINE],
}

impl SpriteGenerator {
    pub fn new() -> Self {
        Self {
            sprite_head: 0,
            scanlines: from_fn(|_| Scanline {
                mask: 0,
                map: [0; 8],
            }),
            sprites: from_fn(|_| Sprite {
                x: 0,
                w: 0,
                pixels: from_fn(|_| Cluster::default()),
                flags: TileFlags::default(),
                tile_id: TileID(0),
            }),
        }
    }

    pub fn reset(&mut self) {
        self.sprite_head = 0;
        for line in &mut self.scanlines {
            line.mask = 0;
        }
    }

    pub fn insert(
        &mut self,
        x: u16,
        y: u16,
        flags: TileFlags,
        tile_id: TileID,
        entry: TileEntry,
        tile: &[Cluster<2>],
        screen_width: u16,
        screen_height: u16
        // vid: &VideoChip,
    ) {
        // Calculate effective sprite dimensions based on rotation
        let (width, height) = if flags.is_rotated() {
            (entry.h as u16, entry.w as u16)
        } else {
            (entry.w as u16, entry.h as u16)
        };

        let sprite = &mut self.sprites[self.sprite_head as usize];
        sprite.x = x;
        sprite.w = width;

        let mut dest_cluster = 0;
        let mut dest_subpixel = 0;
        let mut line_x = x;
        let mut line_y = y;
        let bound_x = x + width;
        for local_y in 0..height {
            for local_x in 0..width {
                // Copy source pixel
                let (tx, ty) = transform_tile_coords(local_x, local_y, width, height, flags);
                let source_index = ((ty * width) + tx) as usize;
                let source_cluster = source_index / PIXELS_PER_CLUSTER as usize;
                let source_subpixel = source_index % PIXELS_PER_CLUSTER as usize;
                let source_pixel = tile[source_cluster].get_subpixel(source_subpixel as u8);

                // Write to destination pixel
                sprite.pixels[dest_cluster].set_subpixel(source_pixel, dest_subpixel as u8);

                // Write bit mask
                let scanline = &mut self.scanlines[line_y as usize];
                let slot = ((line_x as f32 / screen_width as f32) * 8.0) as usize;
                if slot < 8 {
                    // Ensure slot is in valid range
                    // println!("{slot}");
                    scanline.mask |= 1 << slot;
                }

                // Advance
                dest_subpixel += 1;
                if dest_subpixel >= PIXELS_PER_CLUSTER {
                    dest_subpixel = 0;
                    dest_cluster += 1;
                }
                line_x += 1;
                if line_x == bound_x {
                    line_x = x;
                    line_y += 1;
                    if line_y == screen_height {
                        break;
                    }
                }
            }
        }
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
