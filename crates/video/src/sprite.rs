use core::array::from_fn;

use crate::*;

/// A sprite represents a tile's position on the screen and the palette
/// used to draw it.
#[derive(Debug, Clone, Default)]
pub struct SpriteEntry {
    pub y: i16,
    pub x: i16,
    pub id: TileID,
    pub flags: TileFlags,
    pub sub_palette: u8,
}

/// Holds a "presence" mask that helps the iterator figure out if any given slot
/// is occupied by a sprite, and an array containing which sprite entries are visible
/// in this scanline.
#[derive(Debug, Clone)]
pub struct Scanline {
    pub sprite_count: u8,
    // Tracks Which slots contain sprites
    pub mask: u16,
    // Tracks which sprites are visible in this line
    pub sprites: [u8; SPRITES_PER_LINE],
}

/// Manages sprites inserted in a single frame
#[derive(Debug)]
pub struct SpriteGenerator {
    pub sprites: [SpriteEntry; MAX_SPRITES],
    pub scanlines: [Scanline; MAX_LINES],
    sprite_count: u8,
}

impl SpriteGenerator {
    pub fn new() -> Self {
        Self {
            sprites: from_fn(|_| SpriteEntry::default()),
            scanlines: from_fn(|_| Scanline {
                mask: 0,
                sprite_count: 0,
                sprites: Default::default(),
            }),
            sprite_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.sprite_count = 0;
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
        id: TileID,
        sub_palette: PaletteID,
    ) {
        let w = TILE_SIZE as i16;
        let h = TILE_SIZE as i16;

        if self.sprite_count == u8::MAX {
            return;
        }

        if x >= screen_width as i16 || y >= screen_height as i16 {
            return;
        }

        if x <= -w || y <= -h {
            return;
        }

        // Write sprite to sprite bank
        let sprite = &mut self.sprites[self.sprite_count as usize];
        sprite.x = x;
        sprite.y = y;
        sprite.id = id;
        sprite.flags = flags;
        sprite.sub_palette = sub_palette.0;

        // Write sprite index and mask info to scanline
        let min_x = (-x).max(0);
        let min_y = (-y).max(0);
        let max_x = (screen_width as i16 - x).clamp(0, w);
        let max_y = (screen_height as i16 - y).clamp(0, h);
        for local_y in min_y..max_y {
            let screen_y = y + local_y;

            // Acquire scanline ref
            let line = &mut self.scanlines[screen_y as usize];
            if line.sprite_count as usize >= SPRITES_PER_LINE {
                return;
            }
            let local_sprite = line.sprite_count as usize;
            line.sprites[local_sprite] = self.sprite_count;
            line.sprite_count += 1;

            // TODO: No need to iterate all x pixels. Simply determine
            // the range of slots based on min_x and max_x in advance
            // Iterate X pixels to set mask.
            for local_x in min_x..max_x {
                let screen_x = x + local_x;
                // Set bit mask to help iterator skip unused slots
                let slot =
                    ((screen_x as f32 / screen_width as f32) * SLOTS_PER_LINE as f32) as usize;
                debug_assert!(slot < SLOTS_PER_LINE, err!("Invalid slot index"));
                line.mask |= 1 << slot;
            }
        }
        self.sprite_count += 1;
    }
}
