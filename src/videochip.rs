use crate::*;
use core::array::from_fn;

/// A convenient packet of data used to draw a tile as a sprite.
#[derive(Debug, Clone, Copy)]
pub struct DrawBundle {
    pub x: i16,
    pub y: i16,
    pub id: TileID,
    pub flags: TileFlags,
}

/// Main drawing context that manages the screen, tiles, and palette.
#[derive(Debug)]
pub struct VideoChip {
    /// Fixed BG Tilemap
    pub bg: BGMap,
    /// The color rendered if resulting pixel is transparent
    pub bg_color: ColorID,
    /// The main FG palette with 16 colors. Used by sprites.
    pub fg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    /// The main BG palette with 16 colors. Used by BG tiles.
    pub bg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    /// Local Palettes, 16 with 4 ColorIDs each. Each ID referes to a color in the global palette.
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
    /// Maps i16 coordinates into the u8 range, bringing sprites "outside the screen" into view.
    pub wrap_sprites: bool,
    /// Repeats the BG Map outside its borders
    pub wrap_bg: bool,
    /// Offsets the BG Map and Sprite tiles horizontally
    pub scroll_x: i16,
    /// Offsets the BG Map and Sprite tiles vertically
    pub scroll_y: i16,

    // ---------------------- Main Data ----------------------
    pub(crate) sprites: SpriteGenerator,
    // pub(crate) scanlines: [[Cluster<4>; 256 / PIXELS_PER_CLUSTER as usize]; MAX_LINES],
    pub(crate) w: u16,
    pub(crate) h: u16,
    // Pixel data for all tiles, stored as palette indices.
    // Max 64Kb or 256 tiles, whichever runs out first!
    pub(crate) tile_pixels: [Cluster<2>; TILE_MEM_LEN], // 2 bits per pixel
    // ---------------------- Bookkeeping ----------------------
    // view rect cache
    pub(crate) view_left: u16,
    pub(crate) view_top: u16,
    pub(crate) view_right: u16,
    pub(crate) view_bottom: u16,
    // Next available sprite ID.
    tile_id_head: u16,
    // Next available pixel position in the sprite buffer.
    tile_pixel_head: u16,
    // Next available palette.
    palette_head: u8,
}

impl VideoChip {
    /// Creates a new drawing context with default settings.
    pub fn new(w: u16, h: u16) -> Self {
        assert!(
            h > 7 && h <= MAX_LINES as u16,
            err!("Screen height range is 8 to MAX_LINES")
        );

        let mut result = Self {
            bg: BGMap::new(BG_MAX_COLUMNS, BG_MAX_ROWS),
            tile_pixels: [Cluster::default(); TILE_MEM_LEN],
            bg_color: GRAY,
            wrap_sprites: true,
            wrap_bg: true,
            fg_palette: [Color9Bit::default(); COLORS_PER_PALETTE as usize],
            bg_palette: [Color9Bit::default(); COLORS_PER_PALETTE as usize],
            local_palettes: [[ColorID(0); COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
            sprites: SpriteGenerator::new(),
            tile_id_head: 0,
            tile_pixel_head: 0,
            palette_head: 0,
            view_left: 0,
            view_top: 0,
            view_right: w - 1,
            view_bottom: h - 1,
            w,
            h,
            scroll_x: 0,
            scroll_y: 0,
        };
        result.reset_all();

        println!(
            "Total Size of VideoChip:\t{:.1} Kb",
            size_of::<VideoChip>() as f32 / 1024.0
        );
        println!(
            "   Sprite buffers (scanlines):\t{:.1} Kb",
            size_of::<SpriteGenerator>() as f32 / 1024.0
        );
        println!(
            "   Tile Memory:\t\t\t{:.1} Kb",
            (result.tile_pixels.len() * size_of::<Cluster<2>>()) as f32 / 1024.0
        );
        println!(
            "   BG Map:\t\t\t{:.1} Kb",
            size_of::<BGMap>() as f32 / 1024.0
        );

        result
    }

    pub fn max_x(&self) -> u16 {
        self.w - 1
    }

    pub fn max_y(&self) -> u16 {
        self.h - 1
    }

    pub fn width(&self) -> u16 {
        self.w
    }

    pub fn height(&self) -> u16 {
        self.h
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
        self.bg_color = GRAY;
        self.wrap_sprites = true;
        self.reset_scroll();
        self.reset_tiles();
        self.reset_palettes();
        self.reset_bgmap();
        self.reset_viewport();
        self.reset_sprites();
    }

    pub fn reset_tiles(&mut self) {
        self.tile_id_head = 0;
        self.tile_pixel_head = 0;
    }

    pub fn reset_palettes(&mut self) {
        self.fg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                Color9Bit::default()
            }
        });
        self.bg_palette = from_fn(|i| {
            if i < PALETTE_DEFAULT.len() {
                PALETTE_DEFAULT[i]
            } else {
                Color9Bit::default()
            }
        });
        self.local_palettes = from_fn(|_| from_fn(|i| ColorID(i as u8)));
        self.palette_head = 0;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    pub fn reset_bgmap(&mut self) {
        self.bg = BGMap::new(BG_MAX_COLUMNS, BG_MAX_ROWS);
    }

    pub fn reset_viewport(&mut self) {
        self.view_left = 0;
        self.view_top = 0;
        self.view_right = self.max_x();
        self.view_bottom = self.max_y();
    }

    pub fn reset_sprites(&mut self) {
        self.sprites.reset();
    }

    pub fn set_palette(&mut self, index: PaletteID, colors: [ColorID; COLORS_PER_TILE as usize]) {
        debug_assert!(
            index.0 < LOCAL_PALETTE_COUNT,
            err!("Invalid local palette index, must be less than PALETTE_COUNT")
        );
        self.local_palettes[index.0 as usize] = colors;
    }

    pub fn push_subpalette(&mut self, colors: [ColorID; COLORS_PER_TILE as usize]) -> PaletteID {
        assert!(self.palette_head < 16, err!("PALETTE_COUNT exceeded"));
        let result = self.palette_head;
        self.local_palettes[self.palette_head as usize] = colors;
        self.palette_head += 1;
        PaletteID(result)
    }

    pub fn new_tile(&mut self, data: &[u8]) -> TileID {
        assert!(
            data.len() == TILE_PIXEL_COUNT,
            err!("Tile data length must match TILE_PIXEL_COUNT ({})"),
            TILE_PIXEL_COUNT
        );

        let tile_id = self.tile_id_head;
        let pixel_start = self.tile_pixel_head as usize;

        // Check if we have enough space
        if self.tile_id_head == 255 || pixel_start + TILE_PIXEL_COUNT > TILE_MEM_LEN {
            panic!(err!("Not enough space for new tile"))
        }

        // Pack 8 pixels (2 bits each) into each cluster
        let mut cluster_index = self.tile_pixel_head as usize / PIXELS_PER_CLUSTER as usize;
        let mut subpixel_index = 0;
        for i in 0..TILE_PIXEL_COUNT {
            // Clamp color to maximum allowed
            let value = data[i].clamp(0, COLORS_PER_TILE as u8);

            // Set pixel data
            self.tile_pixels[cluster_index].set_subpixel(value, subpixel_index);

            // Advance
            subpixel_index += 1;
            if subpixel_index >= PIXELS_PER_CLUSTER {
                subpixel_index = 0;
                cluster_index += 1;
            }
        }

        self.tile_id_head += 1;
        self.tile_pixel_head += TILE_PIXEL_COUNT as u16;

        TileID(tile_id)
    }

    /// Draws a tile anywhere on the screen using i16 coordinates for convenience. You can
    /// also provide various tile flags, like flipping, and specify a palette id.
    pub fn draw_sprite(&mut self, data: DrawBundle) {
        if data.id.0 >= self.tile_id_head {
            return;
        }
        let size = TILE_SIZE as i16;

        // Handle wrapping
        let wrapped_x: i16;
        let wrapped_y: i16;
        if self.wrap_sprites {
            let screen_x = data.x - self.scroll_x;
            let screen_y = data.y - self.scroll_y;

            wrapped_x = if screen_x < -size {
                screen_x + self.w as i16 + size
            } else if screen_x >= self.w as i16 {
                screen_x - self.w as i16 - size
            } else {
                screen_x
            };

            wrapped_y = if screen_y < -size {
                screen_y + self.h as i16 + size
            } else if screen_y >= self.h as i16 {
                screen_y - self.h as i16 - size
            } else {
                screen_y
            };
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

        let start = data.id.0 as usize * TILE_CLUSTER_COUNT;
        let end = start + TILE_CLUSTER_COUNT;
        let tile = &self.tile_pixels[start..end];

        self.sprites
            .insert(wrapped_x, wrapped_y, self.w, self.h, data.flags, tile);
    }

    /// Increments or decrements an index in a local palette so that its value
    /// cycles between "min" and "max", which represent colors in the Main FG and BG palettes.
    pub fn color_cycle(&mut self, palette: PaletteID, color: u8, min: u8, max: u8) {
        let color_cycle = &mut self.local_palettes[palette.id()][color as usize].0;
        if max > min {
            *color_cycle += 1;
            if *color_cycle > max {
                *color_cycle = min
            }
        } else {
            *color_cycle -= 1;
            if *color_cycle < min {
                *color_cycle = max
            }
        }
    }

    pub fn start_frame(&mut self) {
        self.reset_sprites();
    }

    /// Returns an iterator over the visible screen pixels, yielding RGB colors for each pixel.
    pub fn iter_pixels(&self) -> PixelIter {
        PixelIter::new(self)
    }
}
