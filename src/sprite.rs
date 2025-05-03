use core::array::from_fn;

use crate::*;

/// The part of a sprite that goes into a single scanline
#[derive(Debug, Clone, Default)]
pub struct SpriteSegment {
    pub x: i16,
    pub pixels: Cluster<2>,
    pub palette: PaletteID,
}

/// Holds the sprite segments for a single line, as well as a "presence" mask
/// that helps the iterator figure out if any given slot is occupied by a sprite
#[derive(Debug, Clone)]
pub struct Scanline {
    pub sprite_count: u8,
    // Tracks Which slots contain sprites
    pub mask: u16,
    pub sprites: [SpriteSegment; SPRITES_PER_LINE],
}

/// Facilitates adding a single transformed sprite to multiple scanlines.
#[derive(Debug)]
pub struct SpriteGenerator {
    pub scanlines: [Scanline; MAX_LINES],
}

impl SpriteGenerator {
    pub fn new() -> Self {
        Self {
            scanlines: from_fn(|_| Scanline {
                mask: 0,
                sprite_count: 0,
                sprites: Default::default(),
            }),
        }
    }

    pub fn reset(&mut self) {
        for line in &mut self.scanlines {
            line.mask = 0;
            line.sprite_count = 0;
            // Resetting actual pixels does not seem necessary. Restore if garbage is visible
            // for sprite in &mut line.sprites {
            //     sprite.pixels = Cluster::default();
            // }
        }
    }

    pub fn insert(
        &mut self,
        x: i16,
        y: i16,
        screen_width: u16,
        screen_height: u16,
        flags: TileFlags,
        tile: &[Cluster<2>],
    ) {
        let w = TILE_SIZE as i16;
        let h = TILE_SIZE as i16;

        if x >= screen_width as i16 || y >= screen_height as i16 {
            return;
        }

        if x <= -w || y <= -h {
            return;
        }

        let min_x = (-x).max(0);
        let min_y = (-y).max(0);
        let max_x = (screen_width as i16 - x).clamp(0, w);
        let max_y = (screen_height as i16 - y).clamp(0, h);

        // Copy transformed tile data to scanline buffers
        for local_y in min_y..max_y {
            let screen_y = y + local_y;

            // Acquire scanline ref
            let line = &mut self.scanlines[screen_y as usize];
            if line.sprite_count as usize >= SPRITES_PER_LINE {
                return;
            }

            // Iterate X pixels
            for local_x in min_x..max_x {
                let screen_x = x + local_x;

                // Copy source pixel
                let (tx, ty) = transform_tile_coords(local_x, local_y, w, h, flags);
                let source_index = ((ty * w) + tx) as usize;
                let source_cluster = source_index / PIXELS_PER_CLUSTER as usize;
                let source_subpixel = source_index % PIXELS_PER_CLUSTER as usize;
                let source_pixel = tile[source_cluster].get_subpixel(source_subpixel as u8);

                // Write to destination pixel
                let sprite_index = line.sprite_count as usize;
                let sprite = &mut line.sprites[sprite_index];
                let dest_subpixel = local_x as usize; // max local_x matches cluster size
                sprite
                    .pixels
                    .set_subpixel(source_pixel, dest_subpixel as u8);

                // Set bit mask to help iterator skip unused slots
                let slot =
                    ((screen_x as f32 / screen_width as f32) * SLOTS_PER_LINE as f32) as usize;
                debug_assert!(slot < SLOTS_PER_LINE, err!("Invalid slot index"));
                line.mask |= 1 << slot;

                // This is a little redundant - every new pixel does it!
                // But trying to avoid it creates complexity and is not worth it...
                sprite.palette = flags.palette();
                sprite.x = x;
            }

            line.sprite_count += 1;
        }
    }
}

#[inline(always)]
fn transform_tile_coords(x: i16, y: i16, w: i16, h: i16, flags: TileFlags) -> (i16, i16) {
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
