use core::array::from_fn;

use crate::*;

/// Renders every pixel as it iterates the entire screen.
/// All public fields can be manipulated per line with VideoIRQ!
#[derive(Debug, Clone)]
pub struct PixelIter<'a> {
    vid: &'a VideoChip,
    x: u16,
    y: u16,
    // irq_x: Option<VideoIRQ>, // TODO: Temporarily disabled for performance reasons
    irq_y: Option<VideoIRQ>,

    // Current indices
    wrap_bg: bool,
    subpixel_index: u8,     // Primary counter for background position
    force_bg_color: bool,   // will reuse last bg color when out-of-bounds
    slot_width: f32,        // screen width divided into 16 slots
    bg_cluster: Cluster<2>, // Current pixel cluster
    bg_flags: TileFlags,    // Current background tile flags

    // Stuff that can be manipulated via Horizontal IRQ
    pub current_tile_bank: usize,
    pub current_bg_bank: usize,
    pub tile_banks: [&'a [Tile<2>]; 16],
    pub bg_banks: [&'a Tilemap<BG_LEN>; 16],
    // pub tiles: &'a [Tile<2>],
    // pub bg: &'a Tilemap<BG_LEN>,
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub scanline: Scanline,   // current sprite scanline
    pub bg_color: Color12Bit, // Background color
    pub fg_palette: [Color12Bit; COLORS_PER_PALETTE as usize],
    pub bg_palette: [Color12Bit; COLORS_PER_PALETTE as usize],
    pub local_palettes: [[ColorID; COLORS_PER_TILE as usize]; LOCAL_PALETTE_COUNT as usize],
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (ColorRGB32, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y == self.vid.max_y() as u16 {
            return None;
        }

        // Run X IRQ on every pixel
        // TODO: This hits performance hard! Disabling for now,
        // think of stategies. Maybe per "new bg cluster", instead of
        // every pixel?
        // if let Some(func) = self.irq_x {
        //     func(
        //         self,
        //         self.vid,
        //         self.bg_banks[self.current_bg_bank],
        //         self.tile_banks[self.current_tile_bank],
        //     );
        // }

        let is_outside_viewport = self.x < self.vid.view_left as u16
            || self.x >= self.vid.view_right as u16
            || self.y < self.vid.view_top as u16
            || self.y >= self.vid.view_bottom as u16;

        let color = if is_outside_viewport {
            ColorRGB32::from(self.bg_color)
        } else {
            // Check for foreground pixel, compensating for crop_x
            ColorRGB32::from(self.get_pixel_color())
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
                self.scanline = self.vid.sprite_gen.scanlines[fg_y as usize].clone();
            }
            // Force BG cluster reload on new lines, cache scanline
            if self.y < self.vid.height() {
                // self.scanline = self.vid.sprites.scanlines[(self.y + self.vid.crop_y) as usize].clone();
                reload_cluster = true;
            }
            // Run Y IRQ on every new line
            if let Some(func) = self.irq_y {
                func(
                    self,
                    self.vid,
                    self.bg_banks[self.current_bg_bank],
                    self.tile_banks[self.current_tile_bank],
                );
            }
        }

        // This will be true every few pixels, and once every new line
        if reload_cluster {
            // Previous state - were we outside before?
            let was_outside = self.force_bg_color || self.x == 0;
            let bg = self.bg_banks[self.current_bg_bank];
            self.force_bg_color = !self.wrap_bg && self.is_outside(bg);

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
    pub fn new(vid: &'a VideoChip, tiles: &[&'a [Tile<2>]], bgs: &[&'a Tilemap<BG_LEN>]) -> Self {
        let mut result = Self {
            vid,
            tile_banks: from_fn(|i| if i < tiles.len() { tiles[i] } else { tiles[0] }),
            bg_banks: from_fn(|i| if i < bgs.len() { bgs[i] } else { bgs[0] }),
            current_bg_bank: 0,
            current_tile_bank: 0,
            x: 0,
            y: 0,
            // irq_x: vid.irq_x_callback,
            irq_y: vid.irq_line,

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
            scanline: vid.sprite_gen.scanlines[0].clone(),
        };
        // Check if we're outside the BG map at initialization
        let bg = result.bg_banks[result.current_bg_bank];
        result.force_bg_color = !result.wrap_bg && result.is_outside(bg);
        if !result.force_bg_color {
            result.update_bg_cluster();
        }
        result
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    #[inline]
    fn update_bg_cluster(&mut self) {
        let bg = self.bg_banks[self.current_bg_bank];

        // Calculate effective bg pixel index (which BG pixel this screen pixel "sees")
        let bg_x = (self.x as i16 + self.scroll_x).rem_euclid(bg.width() as i16) as u16;
        let bg_y = (self.y as i16 + self.scroll_y).rem_euclid(bg.height() as i16) as u16;

        // Calculate BG map coordinates
        let bg_col = bg_x / TILE_SIZE as u16;
        let bg_row = bg_y / TILE_SIZE as u16;

        // Get new tile info
        let bg_map_index = (bg_row as usize * bg.columns as usize) + bg_col as usize;
        let current_bg_tile_id = bg.data[bg_map_index].id.0;
        self.bg_flags = bg.data[bg_map_index].flags;

        // Calculate local tile coordinates
        let tile_x = (bg_x % TILE_SIZE as u16) as u8;
        let tile_y = (bg_y % TILE_SIZE as u16) as u8;

        // Get the tile
        let tile_index = current_bg_tile_id as usize;
        let tiles = self.tile_banks[self.current_tile_bank];
        let tile_clusters = &tiles[tile_index].clusters;

        // Get the correct cluster with transformations applied
        // TODO: Update to latest Tile struct, get rid of "from_tile"?
        self.bg_cluster = Cluster::from_tile(tile_clusters, self.bg_flags, tile_y, TILE_SIZE);

        // Calculate subpixel index within the cluster (0-7)
        self.subpixel_index = tile_x % PIXELS_PER_CLUSTER;
    }

    #[inline]
    fn get_pixel_color(&self) -> Color12Bit {
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
                        let w = TILE_SIZE as i16;
                        let h = TILE_SIZE as i16;
                        let sprite_id = self.scanline.sprites[n] as usize;
                        let sprite = &self.vid.sprite_gen.sprites[sprite_id];

                        if fg_x < sprite.x || fg_x >= sprite.x + TILE_SIZE as i16 {
                            continue;
                        }

                        let local_x = fg_x - sprite.x;
                        if local_x >= w || local_x < 0 {
                            continue;
                        }

                        let local_y = self.y as i16 - sprite.y;
                        if local_y >= h || local_y < 0 {
                            continue;
                        }

                        let (tx, ty) = transform_tile_coords(local_x, local_y, w, h, sprite.flags);
                        let tiles = self.tile_banks[self.current_tile_bank];
                        let tile = &tiles[sprite.id.0 as usize];
                        let pixel = tile.get_pixel(tx as u8, ty as u8) as usize;
                        let palette = sprite.flags.palette().id();
                        let pixel = self.vid.local_palettes[palette][pixel].0;
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
    fn is_outside(&self, bg:&Tilemap<BG_LEN>) -> bool {
        // Calculate raw screen position for bounds check
        let raw_x = self.x as i16 + self.scroll_x;
        let raw_y = self.y as i16 + self.scroll_y;

        // Update force_bg_color flag if wrapping is off and pixel is outside BG Map
        let w = bg.width() as i16;
        let h = bg.height() as i16;
        raw_x < 0 || raw_y < 0 || raw_x >= w || raw_y >= h
    }
}
