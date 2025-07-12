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

    // Pre-rendering state
    wrap_bg: bool,
    slot_width: f32, // screen width divided into 16 slots

    // Stuff that can be manipulated via Horizontal IRQ
    pub fg_tile_bank: u8,
    pub bg_tile_bank: u8,
    pub bg_map_bank: u8,
    pub tile_banks: [&'a VideoMemory<TILE_COUNT>; TILE_BANK_COUNT],
    pub bg_banks: [&'a dyn DynamicBGMap; BG_BANK_COUNT],
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub scanline: Scanline, // current sprite scanline
    pub bg_color: RGBA12,   // Background color

    // Dual buffers for parallel processing
    sprite_buffer: [RGBA12; 512], // Sprite layer
    bg_buffer: [RGBA12; 512],     // Background layer (tiles + bg_color)
}

pub struct ScreenCoords {
    pub x: i32,
    pub y: i32,
}

impl<'a> PixelIter<'a> {
    pub fn new(
        vid: &'a VideoChip,
        video_mem: &[&'a VideoMemory<TILE_COUNT>],
        bg_maps: &[&'a dyn DynamicBGMap],
    ) -> Self {
        assert!(
            !video_mem.is_empty(),
            err!("Video Memory bank can't be empty")
        );
        assert!(
            video_mem.len() <= TILE_BANK_COUNT,
            err!("Video Memory bank count ({}) exceeds maximum ({})"),
            video_mem.len(),
            TILE_BANK_COUNT
        );
        assert!(!bg_maps.is_empty(), err!("BG Maps can't be empty"));
        assert!(
            bg_maps.len() <= BG_BANK_COUNT,
            err!("BG Maps count ({}) exceeds maximum ({})"),
            bg_maps.len(),
            BG_BANK_COUNT
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
            bg_banks: from_fn(|i| {
                if i < bg_maps.len() {
                    bg_maps[i]
                } else {
                    bg_maps[0]
                }
            }),
            fg_tile_bank: vid.fg_tile_bank,
            bg_tile_bank: vid.bg_tile_bank,
            bg_map_bank: 0,
            x: 0,
            y: 0,
            // irq_x: vid.irq_x_callback,
            irq_y: vid.irq_line,

            wrap_bg: vid.wrap_bg,
            slot_width: vid.width() as f32 / SLOTS_PER_LINE as f32,

            scroll_x: vid.scroll_x,
            scroll_y: vid.scroll_y,
            bg_color: vid.bg_color,
            scanline: vid.sprite_gen.scanlines[0].clone(),
            sprite_buffer: [RGBA12::BG; 512],
            bg_buffer: [RGBA12::BG; 512],
        };
        // Run Y IRQ on first line before anything else
        result.call_line_irq();

        // Pre-render first line
        result.pre_render_line();
        result
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    #[inline(always)]
    fn call_line_irq(&mut self) {
        if let Some(func) = self.irq_y {
            let bg_map = self.bg_banks[self.bg_map_bank as usize];
            func(self, self.vid, bg_map);
        }
    }

    #[inline(always)]
    fn pre_render_sprites(&mut self, width: u16) {
        // Clear sprite buffer
        for x in 0..width as usize {
            self.sprite_buffer[x] = RGBA12::BG;
        }

        // Early exit if no sprites
        if self.scanline.mask == 0 {
            return;
        }

        let line_y = self.y as i16;
        let bank = self.tile_banks[self.fg_tile_bank as usize];

        // Process sprites from back to front
        for n in (0..self.scanline.sprite_count as usize).rev() {
            let sprite_id = self.scanline.sprites[n] as usize;
            let sprite = &self.vid.sprite_gen.sprites[sprite_id];

            let sprite_y = line_y - sprite.y;
            if sprite_y < 0 || sprite_y >= TILE_SIZE as i16 {
                continue;
            }

            // Calculate sprite bounds
            let start_x = sprite.x.max(0) as usize;
            let end_x = ((sprite.x + TILE_SIZE as i16).min(width as i16)) as usize;

            if start_x >= end_x {
                continue;
            }

            // Check if sprite overlaps any active slots
            let start_slot = (start_x as f32 / self.slot_width) as u16;
            let end_slot = ((end_x - 1) as f32 / self.slot_width) as u16;
            let mut sprite_in_active_slot = false;
            for slot in start_slot..=end_slot {
                if self.scanline.mask & (1 << slot) != 0 {
                    sprite_in_active_slot = true;
                    break;
                }
            }

            if !sprite_in_active_slot {
                continue;
            }

            // Pre-calculate transformation flags
            let flip_x = sprite.flags.is_flipped_x();
            let flip_y = sprite.flags.is_flipped_y();
            let rotated = sprite.flags.is_rotated();
            let tile = &bank.tiles[sprite.id.0 as usize];
            let palette = sprite.flags.palette().id();

            // Render sprite pixels - only in active slots!
            for x in start_x..end_x {
                // Check if this pixel is in an active slot
                let pixel_slot = (x as f32 / self.slot_width) as u16;
                if self.scanline.mask & (1 << pixel_slot) == 0 {
                    continue;
                }

                // Skip if already has a sprite pixel
                if self.sprite_buffer[x].a() > 0 {
                    continue;
                }

                let sprite_x = x as i16 - sprite.x;

                // Inline transform
                let (tx, ty) = if rotated {
                    let rotated_x = TILE_SIZE as i16 - 1 - sprite_y;
                    let rotated_y = sprite_x;
                    if flip_x {
                        (rotated_x, TILE_SIZE as i16 - 1 - rotated_y)
                    } else if flip_y {
                        (TILE_SIZE as i16 - 1 - rotated_x, rotated_y)
                    } else {
                        (rotated_x, rotated_y)
                    }
                } else {
                    let tx = if flip_x {
                        TILE_SIZE as i16 - 1 - sprite_x
                    } else {
                        sprite_x
                    };
                    let ty = if flip_y {
                        TILE_SIZE as i16 - 1 - sprite_y
                    } else {
                        sprite_y
                    };
                    (tx, ty)
                };

                let pixel = tile.get_pixel(tx as u8, ty as u8) as usize;
                let index = bank.sub_palettes[palette][pixel].0;
                let color = bank.palette[index as usize];

                if color.a() > 0 {
                    self.sprite_buffer[x] = color;
                }
            }
        }
    }

    #[inline(always)]
    fn pre_render_background(&mut self, width: u16) {
        let bg = self.bg_banks[self.bg_map_bank as usize];
        let line_y = self.y as i16;
        let bank = self.tile_banks[self.bg_tile_bank as usize];

        // Cache tile data
        let mut cached_bg_col = u16::MAX;
        let mut cached_bg_flags = TileFlags::default();
        let mut cached_bg_cluster = Cluster::<2>::default();

        // Process background
        for x in 0..width {
            // Check viewport
            let in_viewport = x >= self.vid.view_left && x <= self.vid.view_right;

            if !in_viewport {
                self.bg_buffer[x as usize] = self.bg_color;
                continue;
            }

            // Calculate effective bg pixel position
            let bg_x = (x as i16 + self.scroll_x).rem_euclid(bg.width() as i16) as u16;
            let bg_y = (line_y + self.scroll_y).rem_euclid(bg.height() as i16) as u16;

            // Check if outside BG map bounds
            let raw_x = x as i16 + self.scroll_x;
            let raw_y = line_y + self.scroll_y;
            let outside_bg = !self.wrap_bg
                && (raw_x < 0
                    || raw_y < 0
                    || raw_x >= bg.width() as i16
                    || raw_y >= bg.height() as i16);

            if outside_bg {
                self.bg_buffer[x as usize] = self.bg_color;
                continue;
            }

            // Calculate BG map coordinates
            let bg_col = bg_x / TILE_SIZE as u16;
            let bg_row = bg_y / TILE_SIZE as u16;

            // Only recalculate tile data if we've moved to a different tile column
            if bg_col != cached_bg_col {
                cached_bg_col = bg_col;

                let bg_map_index = (bg_row as usize * bg.columns() as usize) + bg_col as usize;
                let bg_cell = bg.cells()[bg_map_index];
                cached_bg_flags = bg_cell.flags;
                let bg_tile_id = bg_cell.id.0;

                // Calculate local tile Y coordinate
                let tile_y = (bg_y % TILE_SIZE as u16) as u8;

                // Get the tile cluster
                let tile_clusters = &bank.tiles[bg_tile_id as usize].clusters;
                cached_bg_cluster =
                    Cluster::from_tile(tile_clusters, cached_bg_flags, tile_y, TILE_SIZE);
            }

            // Calculate local tile X coordinate
            let tile_x = (bg_x % TILE_SIZE as u16) as u8;

            // Get background color
            let color_idx = cached_bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER);
            let palette_idx = cached_bg_flags.palette().0 as usize;
            let global_idx = bank.sub_palettes[palette_idx][color_idx as usize].0 as usize;
            let color = bank.palette[global_idx];

            // Store final color
            if cached_bg_flags.is_fg() && color.a() > 0 {
                self.bg_buffer[x as usize] = color;
            } else {
                self.bg_buffer[x as usize] = if color.a() > 0 { color } else { self.bg_color };
            }
        }
    }

    #[inline(always)]
    fn pre_render_line(&mut self) {
        let width = self.vid.width().min(512);

        // Render both buffers independently
        self.pre_render_sprites(width);
        self.pre_render_background(width);

        // Reset x position for iteration
        self.x = 0;
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (RGBA32, ScreenCoords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y == self.vid.max_y() as u16 {
            return None;
        }

        // Composite sprite and background buffers
        let sprite = self.sprite_buffer[self.x as usize];
        let bg = self.bg_buffer[self.x as usize];

        // Check viewport in y
        let in_viewport = self.y >= self.vid.view_top && self.y <= self.vid.view_bottom;

        // Simple alpha check - sprite wins if it has any alpha
        let final_color = if in_viewport {
            if sprite.a() > 0 { sprite } else { bg }
        } else {
            self.bg_color
        };

        // results
        let color = RGBA32::from(final_color);
        let coords = ScreenCoords {
            x: self.x as i32,
            y: self.y as i32,
        };

        // Increment screen position
        self.x += 1;

        // Check if we need to go to the next line
        if self.x == self.vid.width() {
            self.x = 0;
            self.y += 1;

            // Prepare next line if not at end
            if self.y < self.vid.max_y() as u16 {
                // Cache scanline
                let fg_y = self.y as usize;
                if fg_y < MAX_LINES {
                    self.scanline = self.vid.sprite_gen.scanlines[fg_y as usize].clone();
                }
                // Run Y IRQ for the new line
                self.call_line_irq();
                // Pre-render the new line
                self.pre_render_line();
            }
        }

        // Return the pixel color
        Some((color, coords))
    }
}
