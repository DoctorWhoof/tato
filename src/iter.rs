use crate::*;

pub struct PixelIter<'a> {
    vid: &'a VideoChip,
    x: u16,    // Current screen x position
    y: u16,    // Current screen y position
    w: u16,    // End x position (width)
    h: u16,    // End x position (width)
    left: i16, // Scroll x
    top: i16,  // Scroll y
    wrap_bg: bool,

    // Current indices
    current_bg_tile_id: u8, // Current background tile ID
    subpixel_index: usize,  // Primary counter for background position

    // Caching
    force_bg_color: bool,        // will reuse last bg color when out-of-bounds
    current_bg_flags: TileFlags, // Current background tile flags
    bg_color: ColorRGB,          // Background color (cached)
    scanline: Scanline,          // Current scanline data
    bg_cluster: PixelCluster,    // Current pixel cluster
    fg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    bg_palette: [ColorRGB; COLORS_PER_PALETTE as usize],
    local_palettes: [[ColorID; COLORS_PER_TILE as usize]; PALETTE_COUNT as usize],
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> PixelIter<'a> {
    pub fn new(vid: &'a VideoChip) -> Self {
        let mut result = Self {
            vid,
            x: 0,
            y: 0,
            w: vid.width,
            h: vid.height,
            left: vid.scroll_x,
            top: vid.scroll_y,
            wrap_bg: vid.wrap_bg,
            current_bg_tile_id: 0,
            current_bg_flags: TileFlags::default(),
            bg_cluster: PixelCluster::default(),
            subpixel_index: 0,
            bg_color: vid.bg_palette[vid.bg_color.id()],
            fg_palette: vid.fg_palette.clone(),
            bg_palette: vid.bg_palette.clone(),
            local_palettes: vid.local_palettes.clone(),
            scanline: Scanline::default(),
            force_bg_color: false,
        };
        result.update_indices();
        result
    }

    #[inline]
    fn update_indices(&mut self) {
        // Calculate BG position with scrolling
        let scroll_x = self.left.rem_euclid(BG_WIDTH as i16) as u16;
        let scroll_y = self.top.rem_euclid(BG_HEIGHT as i16) as u16;
        let bg_x = (self.x + scroll_x) % BG_WIDTH;
        let bg_y = (self.y + scroll_y) % BG_HEIGHT;

        // Calculate BG map coordinates
        let bg_col = bg_x / TILE_SIZE;
        let bg_row = bg_y / TILE_SIZE;

        // Get new tile info
        let bg_map_index = (bg_row as usize * BG_COLUMNS as usize) + bg_col as usize;
        self.current_bg_tile_id = self.vid.bg_map.tiles[bg_map_index].0;
        self.current_bg_flags = self.vid.bg_map.flags[bg_map_index];

        // Calculate local tile coordinates
        let tile_x = bg_x % TILE_SIZE;
        let tile_y = bg_y % TILE_SIZE;
        let local_index = tile_x as usize + (tile_y as usize * TILE_SIZE as usize);

        // Get the cluster
        let tile = self.vid.tiles[self.current_bg_tile_id as usize];
        let cluster_index = tile.cluster_index + (local_index / PIXELS_PER_CLUSTER) as u16;
        self.bg_cluster = self.vid.tile_pixels[cluster_index as usize];
        self.subpixel_index = local_index % PIXELS_PER_CLUSTER;
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (ColorRGB, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y >= self.h {
            return None;
        }

        // Cache result coordinates
        let result_coords = ScreenCoords {
            x: self.x as i32,
            y: self.y as i32,
        };

        let is_outside_viewport = self.x < self.vid.view_left
            || self.x >= self.vid.view_left + self.vid.view_width
            || self.y < self.vid.view_top
            || self.y >= self.vid.view_top + self.vid.view_height;

        let color = if is_outside_viewport {
            self.bg_color
        } else {
            // Check for foreground pixel
            let fg_pixel = if self.scanline.bit_slots > 0 {
                if let Some(query) = self.scanline.get(self.x) {
                    let color_idx = query.pixel.0 as usize;
                    let palette = query.flags.palette().0 as usize;
                    self.local_palettes[palette][color_idx].0
                } else {
                    0
                }
            } else {
                0
            };

            // Get color - FG has priority if not transparent
            if fg_pixel > 0 {
                self.fg_palette[fg_pixel as usize]
            } else if self.force_bg_color {
                // Use background color if we're outside bounds
                self.bg_color
            } else {
                // Get pixel from current cluster
                let color = self.bg_cluster.get_subpixel(self.subpixel_index).0;

                // If transparent, use background color
                if color == 0 {
                    self.bg_color
                } else {
                    let palette = self.current_bg_flags.palette().0 as usize;
                    let global_idx = self.local_palettes[palette][color as usize].0 as usize;
                    self.bg_palette[global_idx]
                }
            }
        };

        // Increment screen position
        self.x += 1;

        // Increment subpixel index - this is our primary counter
        self.subpixel_index += 1;

        // Check if we need a new cluster (crossed cluster boundary)
        let mut reload_cluster = false;
        if self.subpixel_index >= PIXELS_PER_CLUSTER {
            reload_cluster = true;
        }

        // Check if we need to go to the next line
        if self.x == self.w {
            self.x = 0;
            self.y += 1;
            if self.y < self.h {
                // Update scanline for new row
                self.scanline = self.vid.sprite_grid.lines[self.y as usize].clone();
                // Reset and reload for new line
                reload_cluster = true;
            }
        }

        // This will only run every few pixels, and once every new line
        if reload_cluster {
            // Calculate raw screen position for bounds check
            let raw_x = self.x as i16 + self.left;
            let raw_y = self.y as i16 + self.top;

            // Update force_bg_color flag if wrapping is off and pixel is outside BG Map
            self.force_bg_color = !self.wrap_bg
                && (raw_x < 0
                    || raw_x >= BG_WIDTH as i16
                    || raw_y < 0
                    || raw_y >= BG_HEIGHT as i16);

            // Only do tile calculations if we're using the actual background
            if !self.force_bg_color {
                self.update_indices();
            }
        }

        // Return the pixel color
        Some((color, result_coords))
    }
}
