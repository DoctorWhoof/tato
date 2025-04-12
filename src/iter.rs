use crate::*;

pub struct PixelIter<'a> {
    vid: &'a VideoChip,
    x: u16, // Current screen x position
    y: u16, // Current screen y position

    // Current indices
    wrap_bg: bool,
    subpixel_index: u8, // Primary counter for background position

    // Caching
    force_bg_color: bool,        // will reuse last bg color when out-of-bounds
    current_bg_flags: TileFlags, // Current background tile flags
    bg_color: ColorRGB,          // Background color (cached)
    scanline: [PixelCluster<4>; 256 / PIXELS_PER_CLUSTER as usize], // Current scanline data
    bg_cluster: PixelCluster<2>, // Current pixel cluster
    fg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    bg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> PixelIter<'a> {
    pub fn new(vid: &'a VideoChip) -> Self {
        let relative_y = vid.crop_y as usize;
        let mut result = Self {
            vid,
            x: 0,
            y: 0,

            wrap_bg: vid.wrap_bg,
            // current_bg_tile_id: 0,
            current_bg_flags: TileFlags::default(),
            bg_cluster: PixelCluster::default(),
            subpixel_index: 0,
            bg_color: vid.bg_palette[vid.bg_color.id()],
            fg_palette: vid.fg_palette.clone(),
            bg_palette: vid.bg_palette.clone(),
            local_palettes: vid.local_palettes.clone(),
            scanline: vid.scanlines[relative_y].clone(),
            force_bg_color: false,
        };
        // Check if we're outside the BG map at initialization
        let raw_x = result.x as i16 + result.vid.scroll_x + vid.crop_x as i16;
        let raw_y = result.y as i16 + result.vid.scroll_y + vid.crop_y as i16;
        let outside =
            raw_x < 0 || raw_y < 0 || raw_x >= BG_WIDTH as i16 || raw_y >= BG_HEIGHT as i16;
        result.force_bg_color = !result.wrap_bg && outside;

        // Calculate the starting subpixel_index based on scroll position
        if !result.force_bg_color {
            // First update the cluster
            result.update_indices();

            // Then adjust the subpixel_index based on scroll_x
            // This ensures we start with the correct offset within the cluster
            let bg_x = (result.x as i16 + result.vid.scroll_x as i16 + vid.crop_x as i16)
                .rem_euclid(BG_WIDTH as i16) as u16;
            let tile_x = bg_x % TILE_SIZE as u16;
            let local_index = tile_x as usize % TILE_SUBPIXELS as usize;
            result.subpixel_index = local_index as u8;
        }
        result
    }

    #[inline]
    fn update_indices(&mut self) {
        // Calculate bg "pixel" index
        let bg_x = (self.x as i16 + self.vid.scroll_x as i16 + self.vid.crop_x as i16)
            .rem_euclid(BG_WIDTH as i16) as u16;
        let bg_y = (self.y as i16 + self.vid.scroll_y as i16 + self.vid.crop_x as i16)
            .rem_euclid(BG_HEIGHT as i16) as u16;

        // Calculate BG map coordinates
        let bg_col = bg_x / TILE_SIZE as u16;
        let bg_row = bg_y / TILE_SIZE as u16;

        // Get new tile info
        let bg_map_index = (bg_row as usize * BG_COLUMNS as usize) + bg_col as usize;
        let current_bg_tile_id = self.vid.bg_map.tiles[bg_map_index].0;
        self.current_bg_flags = self.vid.bg_map.flags[bg_map_index];

        // Calculate local tile coordinates
        let tile_x = bg_x % TILE_SIZE as u16;
        let tile_y = bg_y % TILE_SIZE as u16;

        // Calculate pixel index within the tile
        let local_index = tile_x as usize + (tile_y as usize * TILE_SIZE as usize);

        // Get the cluster - IMPORTANT: Now we're using 8 pixels per cluster!
        let tile = self.vid.tiles[current_bg_tile_id as usize];
        let cluster_index =
            tile.cluster_index as usize + (local_index / PIXELS_PER_CLUSTER as usize);
        self.bg_cluster = self.vid.tile_pixels[cluster_index];

        // Get subpixel index within the cluster (0-7)
        self.subpixel_index = (local_index % PIXELS_PER_CLUSTER as usize) as u8;
    }

    #[inline]
    fn get_pixel_color(&self) -> ColorRGB {
        if self.current_bg_flags.is_fg() && !self.force_bg_color {
            let bg_palette = self.current_bg_flags.palette().0 as usize;
            let color = self.bg_cluster.get_subpixel(self.subpixel_index);
            if color > 0 {
                let global_idx = self.local_palettes[bg_palette][color as usize].0 as usize;
                return self.bg_palette[global_idx];
            }
        }

        // Render sprite, fall back to BG if sprite is zero
        let relative_x = (self.x as usize).saturating_add(self.vid.crop_x as usize);
        let x_cluster = relative_x / PIXELS_PER_CLUSTER as usize;
        let sub_index = (relative_x % PIXELS_PER_CLUSTER as usize) as u8;
        let fg_pixel = {
            let fg_cluster = self.scanline[x_cluster];
            fg_cluster.get_subpixel(sub_index)
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
            if color == 0 {
                self.bg_color
            } else {
                let bg_palette = self.current_bg_flags.palette().0 as usize;
                let global_idx = self.local_palettes[bg_palette][color as usize].0 as usize;
                self.bg_palette[global_idx]
            }
        }
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (ColorRGB, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y == self.vid.max_y as u16 {
            return None;
        }
        // Cache result coordinates
        let result_coords = ScreenCoords {
            x: self.x as i32,
            y: self.y as i32,
        };

        let is_outside_viewport = self.x < self.vid.view_left as u16
            || self.x >= self.vid.view_right as u16
            || self.y < self.vid.view_top as u16
            || self.y >= self.vid.view_bottom as u16;

        let color = if is_outside_viewport {
            self.bg_color
        } else {
            // Check for foreground pixel, compensating for crop_x
            self.get_pixel_color()
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
        if self.x == self.vid.max_x as u16 {
            self.x = 0;
            self.y += 1;
            if self.y < self.vid.max_y as u16 {
                // Cache scanline, compensating for crop_y
                let relative_y = (self.y as usize).saturating_add(self.vid.crop_y as usize);
                self.scanline = self.vid.scanlines[relative_y].clone();
                reload_cluster = true;
            }
        }

        // This will only run every few pixels, and once every new line
        if reload_cluster {
            // Calculate raw screen position for bounds check
            let raw_x = self.x as i16 + self.vid.scroll_x + self.vid.crop_x as i16;
            let raw_y = self.y as i16 + self.vid.scroll_y + self.vid.crop_y as i16;

            // Previous state - were we outside before?
            let was_outside = self.force_bg_color;

            // Update force_bg_color flag if wrapping is off and pixel is outside BG Map
            let w = BG_WIDTH as i16;
            let h = BG_HEIGHT as i16;
            let outside = raw_x < 0 || raw_y < 0 || raw_x >= w || raw_y >= h;

            self.force_bg_color = !self.wrap_bg && outside;

            // Only do tile calculations if we're using the actual background
            if !self.force_bg_color || was_outside {
                self.update_indices();
            }
        }

        // Return the pixel color
        Some((color, result_coords))
    }
}
