use crate::*;

/// Renders every pixel as it iterates the entire screen.
/// All public fields can be manipulated per line with HorizontalIRQ!
pub struct PixelIter<'a> {
    tiles: &'a [Tile<2>],
    vid: &'a VideoChip,
    x: u16, // Current screen x position
    y: u16, // Current screen y position
    horizontal_irq_position: u16,
    horizontal_irq: Option<HorizontalIRQ>,

    // Current indices
    wrap_bg: bool,
    subpixel_index: u8,     // Primary counter for background position
    force_bg_color: bool,   // will reuse last bg color when out-of-bounds
    slot_width: f32,        // screen width divided into 16 slots
    bg_cluster: Cluster<2>, // Current pixel cluster
    bg_flags: TileFlags,    // Current background tile flags

    // Stuff that can be manipulated via Horizontal IRQ
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub scanline: Scanline,  // current sprite scanline
    pub bg_color: Color9Bit, // Background color (cached)
    pub fg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    pub bg_palette: [Color9Bit; COLORS_PER_PALETTE as usize],
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (ColorRGB24, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y == self.vid.max_y() as u16 {
            return None;
        }

        if self.x == self.horizontal_irq_position {
            if let Some(func) = self.horizontal_irq {
                func(self, self.vid, self.y);
            }
        }

        let is_outside_viewport = self.x < self.vid.view_left as u16
            || self.x >= self.vid.view_right as u16
            || self.y < self.vid.view_top as u16
            || self.y >= self.vid.view_bottom as u16;

        let color = if is_outside_viewport {
            ColorRGB24::from(self.bg_color)
        } else {
            // Check for foreground pixel, compensating for crop_x
            ColorRGB24::from(self.get_pixel_color())
        };

        // // Cache result coordinates
        let result_coords = ScreenCoords {
            x: self.x as i32,
            y: self.y as i32,
        };

        // Increment screen position
        self.x += 1;

        // Increment subpixel index - this is our primary counter
        self.subpixel_index += 1;

        // Check if we need a new cluster (crossed cluster boundary)
        let mut reload_cluster = false;
        if self.subpixel_index >= PIXELS_PER_CLUSTER {
            reload_cluster = true;
            self.subpixel_index = 0;
        }

        // Check if we need to go to the next line
        if self.x == self.vid.width() {
            self.x = 0;
            self.y += 1;
            // Cache scanline, compensating for crop_y
            let fg_y = self.y as usize;
            if fg_y < MAX_LINES {
                self.scanline = self.vid.sprites.scanlines[fg_y as usize].clone();
                // self.scanline = self.vid.sprites.scanlines[fg_y as usize].clone();
            }
            // Force BG cluster reload on new lines, cache scanline
            if self.y < self.vid.height() {
                // self.scanline = self.vid.sprites.scanlines[(self.y + self.vid.crop_y) as usize].clone();
                reload_cluster = true;
            }
        }

        // This will be true every few pixels, and once every new line
        if reload_cluster {
            // Previous state - were we outside before?
            let was_outside = self.force_bg_color || self.x == 0;

            self.force_bg_color = !self.wrap_bg && self.is_outside();

            // Only do tile calculations if we're using the actual background
            if !self.force_bg_color || was_outside {
                self.update_bg_cluster();
            }
        }

        // Return the pixel color
        Some((color, result_coords))
    }
}

impl<'a> PixelIter<'a> {
    pub fn new(vid: &'a VideoChip, tiles:&'a [Tile<2>]) -> Self {
        let mut result = Self {
            tiles,
            vid,
            x: 0,
            y: 0,
            horizontal_irq_position: vid.horizontal_irq_position,
            horizontal_irq: vid.horizontal_irq_callback,

            wrap_bg: vid.wrap_bg,
            force_bg_color: false,
            slot_width: vid.width() as f32 / SLOTS_PER_LINE as f32,
            bg_cluster: Cluster::default(),
            bg_flags: TileFlags::default(),

            scroll_x: vid.scroll_x,
            scroll_y: vid.scroll_y,
            subpixel_index: 0,
            bg_color: vid.bg_palette[vid.bg_color.id()],
            fg_palette: vid.fg_palette.clone(),
            bg_palette: vid.bg_palette.clone(),
            local_palettes: vid.local_palettes.clone(),
            scanline: vid.sprites.scanlines[0].clone(),
        };
        // Check if we're outside the BG map at initialization
        result.force_bg_color = !result.wrap_bg && result.is_outside();
        if !result.force_bg_color {
            result.update_bg_cluster();
        }
        result
    }

    #[inline]
    fn update_bg_cluster(&mut self) {
        // Calculate effective bg pixel index (which BG pixel this screen pixel "sees")
        let bg_x = (self.x as i16 + self.scroll_x).rem_euclid(self.vid.bg.width() as i16) as u16;
        let bg_y = (self.y as i16 + self.scroll_y).rem_euclid(self.vid.bg.height() as i16) as u16;

        // Calculate BG map coordinates
        let bg_col = bg_x / TILE_SIZE as u16;
        let bg_row = bg_y / TILE_SIZE as u16;

        // Get new tile info
        let bg_map_index = (bg_row as usize * self.vid.bg.columns as usize) + bg_col as usize;
        let current_bg_tile_id = self.vid.bg.tiles[bg_map_index].0;
        self.bg_flags = self.vid.bg.flags[bg_map_index];

        // Calculate local tile coordinates
        let tile_x = (bg_x % TILE_SIZE as u16) as u8;
        let tile_y = (bg_y % TILE_SIZE as u16) as u8;

        // Get the tile
        let tile_index = current_bg_tile_id as usize;
        let tile_clusters = &self.tiles[tile_index].clusters;

        // Get the correct cluster with transformations applied
        // TODO: Update to latest Tile struct, get rid of "from_tile"?
        self.bg_cluster = Cluster::from_tile(tile_clusters, self.bg_flags, tile_y, TILE_SIZE);

        // Calculate subpixel index within the cluster (0-7)
        self.subpixel_index = tile_x % PIXELS_PER_CLUSTER;
    }

    #[inline]
    fn get_pixel_color(&self) -> Color9Bit {
        // If BG Tile is set to FG and is not zero, return early
        if self.bg_flags.is_fg() && !self.force_bg_color {
            let bg_palette = self.bg_flags.palette().0 as usize;
            let color = self.bg_cluster.get_subpixel(self.subpixel_index);
            if color > 0 {
                let global_idx = self.local_palettes[bg_palette][color as usize].0 as usize;
                return self.bg_palette[global_idx];
            }
        }

        // Render sprite, fall back to BG if sprite is zero
        let fg_pixel = {
            let mut result = 0;
            if self.scanline.mask > 0 {
                let fg_x = self.x as i16;
                let slot = (fg_x as f32 / self.slot_width) as u16;
                // Test slot mask
                if self.scanline.mask & (1 << slot) != 0 {
                    // Iterate sprites in line
                    'sprite_loop: for n in (0..self.scanline.sprite_count as usize).rev() {
                        let sprite = &self.scanline.sprites[n];
                        if fg_x < sprite.x || fg_x >= sprite.x + TILE_SIZE as i16 {
                            continue;
                        }

                        // let (tx, ty) = transform_tile_coords(local_x, local_y, w, h, flags);
                        // let source_pixel = tile.get_pixel(tx as u8, ty as u8);

                        let color_index =
                            sprite.pixels.get_subpixel((fg_x - sprite.x) as u8) as usize;
                        let pixel = self.vid.local_palettes[sprite.palette.id()][color_index].0;
                        if pixel > 0 {
                            result = pixel;
                            break 'sprite_loop;
                        }
                    }
                }
            }
            result
        };

        // Get color - FG has priority if not transparent
        if fg_pixel > 0 {
            self.fg_palette[fg_pixel as usize]
        } else if self.force_bg_color {
            // Use background color if we're outside bounds
            self.bg_color
        } else {
            // Get pixel from current cluster
            let color = self.bg_cluster.get_subpixel(self.subpixel_index);
            // If transparent, use background color
            let bg_palette = self.bg_flags.palette().0 as usize;
            let global_idx = self.local_palettes[bg_palette][color as usize].0 as usize;
            if global_idx == 0 {
                self.bg_color
            } else {
                self.bg_palette[global_idx]
            }
        }
    }

    #[inline(always)]
    fn is_outside(&self) -> bool {
        // Calculate raw screen position for bounds check
        let raw_x = self.x as i16 + self.scroll_x;
        let raw_y = self.y as i16 + self.scroll_y;

        // Update force_bg_color flag if wrapping is off and pixel is outside BG Map
        let w = self.vid.bg.width() as i16;
        let h = self.vid.bg.height() as i16;
        raw_x < 0 || raw_y < 0 || raw_x >= w || raw_y >= h
    }
}
