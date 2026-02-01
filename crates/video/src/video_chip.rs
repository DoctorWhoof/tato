use tato_math::Vec2;

use crate::*;

/// A convenient packet of data used to draw a single tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: i16,
    pub y: i16,
    pub id: TileID,
    pub flags: TileFlags,
    pub colors: Palette,
}

/// A convenient packet of data used to draw a tilemap as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct SpriteBundle {
    pub x: i16,
    pub y: i16,
    pub flip_x: bool,
    pub flip_y: bool,
    pub tile_offset: u8,
    pub palette_override: Option<Palette>,
}

/// Main drawing context that manages the screen, tiles, and palette.
#[derive(Debug)]
pub struct VideoChip {
    /// The color rendered if resulting pixel is transparent
    pub bg_color: RGBA12,
    /// The color outside the viewport
    pub crop_color: RGBA12,
    /// Brings sprites "outside the screen" into view.
    pub wrap_sprites: bool,
    /// Repeats the BG Map outside its borders
    pub wrap_bg: bool,
    /// Offsets the BG Map and Sprite tiles horizontally
    pub scroll_x: i16,
    /// Offsets the BG Map and Sprite tiles vertically
    pub scroll_y: i16,
    // ---------------------- Iterator control ----------------------
    /// A callback that can modify the iterator, called once per line.
    /// It is automatically passed to the PixelIterator.
    // pub irq_x_callback: Option<VideoIRQ>,
    pub irq_line: Option<VideoIRQ>,
    pub fg_tile_bank: u8,
    pub bg_tile_bank: u8,
    // ---------------------- Main Data ----------------------
    pub(crate) sprite_gen: SpriteGenerator,
    pub(crate) w: u16,
    pub(crate) h: u16,
    // ---------------------- Bookkeeping ----------------------
    // view rect cache
    pub(crate) view_left: u16,
    pub(crate) view_top: u16,
    pub(crate) view_right: u16,
    pub(crate) view_bottom: u16,
    // Internal timer.
    pub frame_number: usize,
}

impl VideoChip {
    /// Creates a new drawing context with default settings.
    pub fn new(w: u16, h: u16) -> Self {
        assert!(h > 7 && h <= MAX_RESOLUTION_Y as u16, err!("Screen height range is 8 to MAX_RESOLUTION_Y"));

        let mut result = Self {
            bg_color: RGBA12::BLACK,
            crop_color: RGBA12::BLACK,
            wrap_sprites: false,
            wrap_bg: false,
            sprite_gen: SpriteGenerator::new(),
            view_left: 0,
            view_top: 0,
            view_right: w - 1,
            view_bottom: h - 1,
            w,
            h,
            scroll_x: 0,
            scroll_y: 0,
            frame_number: 0,
            // Video IRQs
            // irq_x_callback: None,
            irq_line: None,
            fg_tile_bank: 0,
            bg_tile_bank: 0,
        };
        result.reset_all();

        // println!(
        //     "Total Size of VideoChip:\t{:.1} Kb",
        //     size_of::<VideoChip>() as f32 / 1024.0
        // );
        // println!(
        //     "   Sprite buffers (scanlines):\t{:.1} Kb",
        //     size_of::<SpriteGenerator>() as f32 / 1024.0
        // );

        result
    }

    #[inline]
    pub fn max_x(&self) -> u16 {
        self.w - 1
    }

    #[inline]
    pub fn max_y(&self) -> u16 {
        self.h - 1
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.w
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.h
    }

    #[inline]
    pub fn size(&self) -> Vec2<i16> {
        Vec2 { x: self.w as i16, y: self.h as i16 }
    }

    #[inline]
    pub fn frame_number(&self) -> usize {
        self.frame_number
    }

    /// Does not affect BG or Sprites calculation, but "masks" PixelIter pixels outside
    /// this rectangular area with the BG Color
    pub fn set_viewport(&mut self, left: u16, top: u16, w: u16, h: u16) {
        self.view_left = left;
        self.view_top = top;
        self.view_right = left.saturating_add(w);
        self.view_bottom = top.saturating_add(h);
    }

    /// Resets the chip to its initial state.
    pub fn reset_all(&mut self) {
        self.bg_color = RGBA12::BLACK;
        self.wrap_sprites = false;
        self.wrap_bg = false;
        self.frame_number = 0;
        self.fg_tile_bank = 0;
        self.bg_tile_bank = 0;
        self.reset_scroll();
        self.reset_viewport();
        self.reset_sprites();
        self.irq_line = None;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    pub fn reset_viewport(&mut self) {
        self.view_left = 0;
        self.view_top = 0;
        self.view_right = self.max_x();
        self.view_bottom = self.max_y();
    }

    pub fn reset_sprites(&mut self) {
        self.sprite_gen.reset();
    }

    /// Flips a coordinate based on the axis length (the length in rows or columns
    /// of the the sprite's Tilemap in that axis) and flip state.
    #[inline(always)]
    fn flip_tile_coords(coord: i16, axis_length: i16, state: bool) -> i16 {
        if state { axis_length - coord - 1 } else { coord }
    }

    /// A sprite is in reality a Tilemap, since it is a collection of tiles! This function
    /// will draw all the tiles in the FG layer at cordinates x and y.
    pub fn draw_sprite(&mut self, bundle: SpriteBundle, sprite: &dyn DynTilemap) {
        for row in 0..sprite.rows() as i16 {
            for col in 0..sprite.columns() as i16 {
                let Some(cell) = sprite.get_cell(col, row) else {
                    continue;
                };

                let draw_col = Self::flip_tile_coords(col, sprite.columns() as i16, bundle.flip_x);
                let draw_row = Self::flip_tile_coords(row, sprite.rows() as i16, bundle.flip_y);

                let mut flags = cell.flags;
                if bundle.flip_x {
                    flags = flags.toggle_flip_x();
                };
                if bundle.flip_y {
                    flags = flags.toggle_flip_y();
                };

                self.draw_fg_tile(DrawBundle {
                    x: (draw_col as i16 * TILE_SIZE as i16) + bundle.x,
                    y: (draw_row as i16 * TILE_SIZE as i16) + bundle.y,
                    id: cell.id,
                    flags,
                    colors: bundle.palette_override.unwrap_or(cell.colors),
                });
            }
        }
    }

    /// Draws a tile anywhere on the screen using i16 coordinates for convenience. You can
    /// also provide various tile flags, like flipping, and specify a palette id.
    pub fn draw_fg_tile(&mut self, data: DrawBundle) {
        let size = TILE_SIZE as i16;

        // Handle wrapping
        let wrapped_x: i16;
        let wrapped_y: i16;
        if self.wrap_sprites {
            let screen_x = data.x - self.scroll_x;
            let screen_y = data.y - self.scroll_y;

            let w = self.w as i16;
            let h = self.h as i16;
            let size = TILE_SIZE as i16;

            let adjusted_x = screen_x + size;
            let adjusted_y = screen_y + size;

            // Apply proper modulo wrapping
            let wrapped_adjusted_x = ((adjusted_x % (w + size * 2)) + (w + size * 2)) % (w + size * 2);
            let wrapped_adjusted_y = ((adjusted_y % (h + size * 2)) + (h + size * 2)) % (h + size * 2);

            // Adjust back to get the final coordinates
            wrapped_x = wrapped_adjusted_x - size;
            wrapped_y = wrapped_adjusted_y - size;
        } else {
            let max_x = self.scroll_x + self.max_x() as i16;
            if data.x + size < self.scroll_x || data.x > max_x {
                return;
            } else {
                wrapped_x = data.x - self.scroll_x;
            }
            let max_y = self.scroll_y + self.max_y() as i16;
            if data.y + size < self.scroll_y || data.y > max_y {
                return;
            } else {
                wrapped_y = data.y - self.scroll_y;
            }
        }

        self.sprite_gen.insert(wrapped_x, wrapped_y, self.w, self.h, data.flags, data.id, data.colors);
    }

    pub fn frame_start(&mut self, is_paused: bool) {
        if is_paused {
            return;
        }
        // Frame count pauses just in case someone uses the frame count to
        // process animations (not recommended...)
        self.frame_number += 1;
        self.reset_sprites();
    }

    /// Returns an iterator over the visible screen pixels, yielding RGB colors for each pixel.
    /// Requires a reference to the Tile array and one for the BG Tilemap array.
    pub fn iter_pixels<'a, T>(&'a self, video_banks: &'a [Bank], tilemaps: &'a [&'a T]) -> PixelIter<'a>
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        PixelIter::new(self, video_banks, tilemaps)
    }
}
