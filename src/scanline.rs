use core::array::from_fn;

use crate::*;

const MAX_TILE_LEN: usize = MAX_TILE_SIZE as usize * MAX_TILE_SIZE as usize;
const CLUSTER_COUNT: usize = MAX_TILE_LEN / PIXELS_PER_CLUSTER as usize;

#[derive(Debug, Clone)]
pub(crate) struct Sprite {
    x: u16,
    w: u16,
    pixels: [Cluster<2>; CLUSTER_COUNT],
}

pub struct Scanline {
    mask: u16,
    map: [u8; 8], // each slot can refer to 8 sprite indices (1 bit per sprite)
}

pub struct SpriteGenerator {
    sprite_head: u8,
    scanlines: [Scanline; 256],
    sprites: [Sprite; 8],
}

impl SpriteGenerator {
    pub fn reset(&mut self) {
        self.sprite_head = 0;
        for line in &mut self.scanlines {
            line.mask = 0;
        }
    }

    pub fn insert_sprite(
        &mut self,
        x: u16,
        y: u16,
        flags: TileFlags,
        tile_id: TileID,
        vid: &VideoChip,
    ) {
        let entry = vid.tiles[tile_id.0 as usize];

        // Calculate effective sprite dimensions based on rotation
        let (width, height) = if flags.is_rotated() {
            (entry.h as u16, entry.w as u16)
        } else {
            (entry.w as u16, entry.h as u16)
        };

        let start = entry.cluster_index as usize;
        let end = start + (entry.w as usize * entry.h as usize) / PIXELS_PER_CLUSTER as usize;
        let tile = &vid.tile_pixels[start..end];

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
                let slot = ((line_x as f32 / vid.width() as f32) * 16.0) as usize;
                if slot < 16 {  // Ensure slot is in valid range
                    scanline.mask |= 1 << slot;
                }

                // Advance
                dest_subpixel += 1;
                if dest_subpixel == PIXELS_PER_CLUSTER {
                    dest_cluster += 1;
                }
                line_x += 1;
                if line_x == bound_x {
                    line_x = x;
                    line_y += 1;
                    if line_y as u32 == vid.height() {
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
