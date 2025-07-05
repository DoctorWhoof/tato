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
    pub fg_tile_bank: u8,
    pub bg_tile_bank: u8,
    pub bg_map_bank: u8,
    pub tile_banks: [&'a VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    pub bg_banks: [Option<&'a dyn DynamicBGMap>; BG_BANK_COUNT], // BG map banks are optional
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub scanline: Scanline, // current sprite scanline
    pub bg_color: RGBA12,   // Background color
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> PixelIter<'a> {
    pub fn new(
        vid: &'a VideoChip,
        video_mem: [&'a VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
        bg_maps: [Option<&'a dyn DynamicBGMap>; BG_BANK_COUNT],
    ) -> Self {
        assert!(
            !video_mem.is_empty(),
            err!("Video Memory bank can't be empty")
        );
        let mut result = Self {
            vid,
            tile_banks: from_fn(|i| {
                if i < video_mem.len() {
                    video_mem[i]
                } else {
                    video_mem[0]
                }
            }),
            bg_banks: bg_maps,
            fg_tile_bank: vid.fg_tile_bank,
            bg_tile_bank: vid.bg_tile_bank,
            bg_map_bank: 0,
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
            bg_color: vid.bg_color,
            scanline: vid.sprite_gen.scanlines[0].clone(),
        };
        // Run Y IRQ on first line before anything else
        result.call_line_irq();
        // Check if we're outside the BG map at initialization
        if let Some(bg) = result.bg_banks[result.bg_map_bank as usize] {
            result.force_bg_color = !result.wrap_bg && result.is_outside(bg);
            // Update bg data before first pixel is rendered
            result.update_bg_cluster();
        } else {
            result.force_bg_color = true;
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
    fn call_line_irq(&mut self) {
        if let Some(func) = self.irq_y {
            if let Some(bg_map) = self.bg_banks[self.bg_map_bank as usize] {
                func(self, self.vid, bg_map);
            } else {
                // Create a dummy BGMap for the IRQ call when bg_maps is None
                // This is a workaround - ideally the IRQ signature should be updated
                // to handle None bg_maps case
                return;
            }
        }
    }

    #[inline]
    fn update_bg_cluster(&mut self) {
        let Some(bg) = self.bg_banks[self.bg_map_bank as usize] else {
            return;
        };

        // Calculate effective bg pixel index (which BG pixel this screen pixel "sees")
        let bg_x = (self.x as i16 + self.scroll_x).rem_euclid(bg.width() as i16) as u16;
        let bg_y = (self.y as i16 + self.scroll_y).rem_euclid(bg.height() as i16) as u16;

        // Calculate BG map coordinates
        let bg_col = bg_x / TILE_SIZE as u16;
        let bg_row = bg_y / TILE_SIZE as u16;

        // Get new tile info
        let bg_map_index = (bg_row as usize * bg.columns() as usize) + bg_col as usize;
        let bg_cell = bg.cells()[bg_map_index];
        let current_bg_tile_id = bg_cell.id.0;
        self.bg_flags = bg_cell.flags;

        // Calculate local tile coordinates
        let tile_x = (bg_x % TILE_SIZE as u16) as u8;
        let tile_y = (bg_y % TILE_SIZE as u16) as u8;

        // Get the tile
        let tile_index = current_bg_tile_id as usize;
        let bank = self.tile_banks[self.bg_tile_bank as usize];
        let tile_clusters = &bank.tiles[tile_index].clusters;

        // Get the correct cluster with transformations applied
        // TODO: Update to latest Tile struct, get rid of "from_tile"?
        self.bg_cluster = Cluster::from_tile(tile_clusters, self.bg_flags, tile_y, TILE_SIZE);

        // Calculate subpixel index within the cluster (0-7)
        self.subpixel_index = tile_x % PIXELS_PER_CLUSTER;
    }

    #[inline]
    fn get_pixel_color(&self) -> RGBA12 {
        // If BG Tile is set to FG and is not zero, return early
        if self.bg_flags.is_fg() && !self.force_bg_color && self.bg_banks[self.bg_map_bank as usize].is_some() {
            let sub_palette = self.bg_flags.palette().0 as usize;
            let color = self.bg_cluster.get_subpixel(self.subpixel_index);
            // if color > 0 {
            let bank = self.tile_banks[self.bg_tile_bank as usize];
            let global_idx = bank.sub_palettes[sub_palette][color as usize].0 as usize;
            let color = bank.palette[global_idx];
            if color.a() > 0 {
                return color;
            }
        }

        // Render sprite, fall back to BG if sprite is zero
        let fg_color = {
            let mut result = RGBA12::BG;
            if self.scanline.mask > 0 {
                let fg_x = self.x as i16;
                let slot = (fg_x as f32 / self.slot_width) as u16;
                // Test slot mask
                if self.scanline.mask & (1 << slot) != 0 {
                    // Iterate sprites in line
                    let w = TILE_SIZE as i16;
                    let h = TILE_SIZE as i16;
                    'sprite_loop: for n in (0..self.scanline.sprite_count as usize).rev() {
                        let sprite_id = self.scanline.sprites[n] as usize;
                        let sprite = &self.vid.sprite_gen.sprites[sprite_id];

                        if fg_x < sprite.x || fg_x >= sprite.x + TILE_SIZE as i16 {
                            continue;
                        }

                        // TODO: PERF: Remove these checks
                        let local_x = fg_x - sprite.x;
                        if local_x >= w || local_x < 0 {
                            continue;
                        }
                        let local_y = self.y as i16 - sprite.y;
                        if local_y >= h || local_y < 0 {
                            continue;
                        }

                        let (tx, ty) = transform_tile_coords(local_x, local_y, w, h, sprite.flags);
                        let bank = self.tile_banks[self.fg_tile_bank as usize];
                        let tile = &bank.tiles[sprite.id.0 as usize];
                        let pixel = tile.get_pixel(tx as u8, ty as u8) as usize;
                        let palette = sprite.flags.palette().id();
                        let index = bank.sub_palettes[palette][pixel].0;
                        let color = bank.palette[index as usize];
                        if color.a() > 0 {
                            result = color;
                            break 'sprite_loop;
                        }
                    }
                }
            }
            result
        };

        // Color result - FG has priority if not transparent
        if fg_color.a() > 0 {
            fg_color
        } else if self.force_bg_color || self.bg_banks[self.bg_map_bank as usize].is_none() {
            // Use background color if we're outside bounds or no bg maps
            self.bg_color
        } else {
            // Get pixel from current cluster
            let color = self.bg_cluster.get_subpixel(self.subpixel_index);
            let bg_palette = self.bg_flags.palette().0 as usize;
            let bank = self.tile_banks[self.bg_tile_bank as usize];
            let global_idx = bank.sub_palettes[bg_palette][color as usize].0 as usize;
            let result = bank.palette[global_idx];
            // If transparent, use background color.
            // Will ignore intermediate alpha levels - it's 0 or 1 only!
            // You can still get 8 transparency levels in the bg color itself, though.
            if result.a() == 0 {
                self.bg_color
            } else {
                result
            }
        }
    }

    #[inline(always)]
    fn is_outside(&self, bg: &'a dyn DynamicBGMap) -> bool {
        // Calculate raw screen position for bounds check
        let raw_x = self.x as i16 + self.scroll_x;
        let raw_y = self.y as i16 + self.scroll_y;
        // Update force_bg_color flag if wrapping is off and pixel is outside BG Map
        let w = bg.width() as i16;
        let h = bg.height() as i16;
        raw_x < 0 || raw_y < 0 || raw_x >= w || raw_y >= h
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (RGBA32, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y == self.vid.max_y() as u16 {
            return None;
        }

        // Run X IRQ on every pixel
        // TODO: This hits performance hard! Disabling for now,
        // thinking of strategies. Maybe per "new bg cluster", instead of
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
            || self.y < self.vid.view_top as u16
            || self.x > self.vid.view_right as u16 // allows view.right to be included!
            || self.y > self.vid.view_bottom as u16; // allows view.bottom to be included!

        let color = if is_outside_viewport {
            RGBA32::from(self.bg_color)
        } else {
            // Check for foreground pixel, compensating for crop_x
            RGBA32::from(self.get_pixel_color())
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
            self.call_line_irq();
        }

        // This will be true every few pixels, and once every new line
        if reload_cluster {
            // Previous state - were we outside before?
            let was_outside = self.force_bg_color || self.x == 0;
            if let Some(bg) = self.bg_banks[self.bg_map_bank as usize] {
                self.force_bg_color = !self.wrap_bg && self.is_outside(bg);

                // Only do tile calculations if we're using the actual background
                if !self.force_bg_color || was_outside {
                    self.update_bg_cluster();
                }
            } else {
                self.force_bg_color = true;
            }
        }

        // Return the pixel color
        Some((color, result_coords))
    }
}
