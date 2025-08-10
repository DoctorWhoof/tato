use core::array::from_fn;

use crate::*;

// Z-buffer priority constants for compositing
const Z_BG_COLOR: u8 = 0;      // Background color (lowest priority)
const Z_BG_TILE: u8 = 1;       // Normal background tiles
const Z_SPRITE: u8 = 2;        // Sprites
const Z_BG_FOREGROUND: u8 = 3; // Background tiles with is_fg() flag (highest priority)

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
    pub bg_banks: [TilemapRef<'a>; BG_BANK_COUNT],
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub scanline: Scanline, // current sprite scanline
    pub bg_color: RGBA12,   // Background color

    // Dual buffers for parallel processing
    sprite_buffer: [RGBA12; 512], // Sprite layer
    bg_buffer: [RGBA12; 512],     // Background layer (tiles + bg_color)
}

#[derive(Debug, Clone, Copy)]
pub struct Coords {
    pub x: i16,
    pub y: i16,
}

impl<'a> PixelIter<'a> {
    pub fn new<T>(
        vid: &'a VideoChip,
        video_mem: &[&'a VideoMemory<TILE_COUNT>],
        bg_maps: &[&'a T],
    ) -> Self
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        assert!(!video_mem.is_empty(), err!("Video Memory bank can't be empty"));
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
            tile_banks: from_fn(|i| if i < video_mem.len() { video_mem[i] } else { video_mem[0] }),
            bg_banks: from_fn(|i| {
                if i < bg_maps.len() { bg_maps[i].into() } else { bg_maps[0].into() }
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
            sprite_buffer: [RGBA12::BG.with_z(Z_SPRITE); 512],
            bg_buffer: [RGBA12::BG.with_z(Z_BG_COLOR); 512],
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
            func(self, self.vid, &bg_map);
        }
    }

    #[inline(always)]
    fn pre_render_sprites(&mut self, width: u16) {
        // Pre-calculate viewport bounds
        let viewport_start = self.vid.view_left.max(0) as usize;
        let viewport_end = (self.vid.view_right + 1).min(width) as usize;

        // Clear sprite buffer only in viewport
        unsafe {
            let ptr = self.sprite_buffer.as_mut_ptr();
            for x in viewport_start..viewport_end {
                *ptr.add(x) = RGBA12::BG;
            }
        }

        // Early exit if no sprites or no viewport
        if self.scanline.mask == 0 || viewport_start >= viewport_end {
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

            // Calculate sprite bounds clamped to viewport
            let sprite_start = sprite.x.max(0) as usize;
            let sprite_end = ((sprite.x + TILE_SIZE as i16).min(width as i16)) as usize;

            // Clamp to viewport
            let start_x = sprite_start.max(viewport_start);
            let end_x = sprite_end.min(viewport_end);

            if start_x >= end_x {
                continue;
            }

            // Check if sprite overlaps any active slots
            let start_slot = (start_x as f32 / self.slot_width) as u16;
            let end_slot = ((end_x - 1) as f32 / self.slot_width) as u16;

            // Quick check if any slots are active in range
            let slot_mask = if end_slot >= start_slot {
                let span = end_slot - start_slot + 1;
                if span >= 16 {
                    !0u16 // All bits set if span covers all slots
                } else {
                    let mask = (1u16 << span) - 1;
                    mask << start_slot
                }
            } else {
                0
            };

            if self.scanline.mask & slot_mask == 0 {
                continue;
            }

            // Pre-calculate transformation flags
            let flip_x = sprite.flags.is_flipped_x();
            let flip_y = sprite.flags.is_flipped_y();
            let rotated = sprite.flags.is_rotated();
            let tile = &bank.tiles[sprite.id.0 as usize];
            let palette = sprite.sub_palette;

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
                    let tx = if flip_x { TILE_SIZE as i16 - 1 - sprite_x } else { sprite_x };
                    let ty = if flip_y { TILE_SIZE as i16 - 1 - sprite_y } else { sprite_y };
                    (tx, ty)
                };

                let pixel = tile.get_pixel(tx as u8, ty as u8) as usize;
                let index = bank.sub_palettes[palette as usize][pixel].0;
                let color = bank.palette[index as usize];

                if color.a() > 0 {
                    self.sprite_buffer[x] = color.with_z(Z_SPRITE);
                }
            }
        }
    }

    #[inline(always)]
    fn pre_render_background(&mut self, width: u16) {
        let bg = self.bg_banks[self.bg_map_bank as usize];
        let line_y = self.y as i16;
        let bank = self.tile_banks[self.bg_tile_bank as usize];

        // Pre-calculate viewport bounds
        let viewport_start = self.vid.view_left.max(0) as usize;
        let viewport_end = (self.vid.view_right + 1).min(width) as usize;

        // Fast fill non-viewport areas with bg_color
        unsafe {
            let bg_color = self.bg_color;
            // Fill start
            if viewport_start > 0 {
                let ptr = self.bg_buffer.as_mut_ptr();
                for i in 0..viewport_start {
                    *ptr.add(i) = bg_color.with_z(Z_BG_COLOR);
                }
            }
            // Fill end
            if viewport_end < width as usize {
                let ptr = self.bg_buffer.as_mut_ptr();
                for i in viewport_end..width as usize {
                    *ptr.add(i) = bg_color.with_z(Z_BG_COLOR);
                }
            }
        }

        // Early exit if no viewport pixels
        if viewport_start >= viewport_end {
            return;
        }

        // Pre-calculate Y coordinates once
        let bg_y_base = line_y + self.scroll_y;
        let bg_height = bg.height() as i16;
        let bg_width = bg.width() as i16;

        // Check Y bounds once for entire line (when wrap_bg is false)
        if !self.wrap_bg && (bg_y_base < 0 || bg_y_base >= bg_height) {
            let bg_color = self.bg_color;
            for x in viewport_start..viewport_end {
                self.bg_buffer[x] = bg_color.with_z(Z_BG_COLOR);
            }
            return;
        }

        let bg_y = bg_y_base.rem_euclid(bg_height) as u16;
        let bg_row = bg_y / TILE_SIZE as u16;
        let tile_y = (bg_y % TILE_SIZE as u16) as u8;
        let bg_columns = bg.columns() as usize;

        // Process viewport pixels in tile-aligned batches
        let mut x = viewport_start;

        while x < viewport_end {
            // Calculate starting BG X coordinate
            let bg_x_base = x as i16 + self.scroll_x;

            // Handle horizontal out of bounds
            if !self.wrap_bg {
                if bg_x_base < 0 {
                    // Skip negative pixels
                    let skip = (-bg_x_base).min((viewport_end - x) as i16) as usize;
                    let bg_color = self.bg_color;
                    for i in 0..skip {
                        self.bg_buffer[x + i] = bg_color.with_z(Z_BG_COLOR);
                    }
                    x += skip;
                    continue;
                } else if bg_x_base >= bg_width {
                    // Fill rest with bg_color and exit
                    let bg_color = self.bg_color;
                    for i in x..viewport_end {
                        self.bg_buffer[i] = bg_color.with_z(Z_BG_COLOR);
                    }
                    break;
                }
            }

            let bg_x = bg_x_base.rem_euclid(bg_width) as u16;
            let bg_col = bg_x / TILE_SIZE as u16;
            let tile_x_start = (bg_x % TILE_SIZE as u16) as u8;

            // Get tile data
            let bg_map_index = (bg_row as usize * bg_columns) + bg_col as usize;
            let bg_cell = bg.cells()[bg_map_index];
            let bg_flags = bg_cell.flags;
            let bg_palette = bg_cell.sub_palette.0 as usize;
            let bg_tile_id = bg_cell.id.0 as usize;

            // Get the tile cluster for this row
            let tile = &bank.tiles[bg_tile_id];
            let bg_cluster = Cluster::from_tile(&tile.clusters, bg_flags, tile_y, TILE_SIZE);

            // Pre-fetch palette data
            let sub_palette = &bank.sub_palettes[bg_palette];
            let palette = &bank.palette;
            let is_fg = bg_flags.is_fg();
            let bg_color = self.bg_color;

            // Calculate pixels to process in this tile
            let tile_pixels_remaining = (TILE_SIZE as usize) - tile_x_start as usize;
            let viewport_pixels_remaining = viewport_end - x;
            let pixels_to_process = tile_pixels_remaining.min(viewport_pixels_remaining);

            // Additional constraint for wrap_bg=false
            let pixels_to_process = if !self.wrap_bg && bg_x_base >= 0 {
                let max_x = (bg_width - bg_x_base) as usize;
                pixels_to_process.min(max_x)
            } else {
                pixels_to_process
            };

            // Process pixels in batch
            unsafe {
                let dst_ptr = self.bg_buffer.as_mut_ptr().add(x);

                // Unroll by 4 when possible
                let chunks = pixels_to_process / 4;
                let remainder = pixels_to_process % 4;

                for chunk in 0..chunks {
                    let base_idx = chunk * 4;
                    for i in 0..4 {
                        let tile_x = tile_x_start + (base_idx + i) as u8;
                        let color_idx =
                            bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER) as usize;
                        let global_idx = sub_palette[color_idx].0 as usize;
                        let color = palette[global_idx];

                        let final_color = if color.a() > 0 {
                            let z_value = if is_fg { Z_BG_FOREGROUND } else { Z_BG_TILE };
                            color.with_z(z_value)
                        } else {
                            bg_color.with_z(Z_BG_COLOR)
                        };

                        *dst_ptr.add(base_idx + i) = final_color;
                    }
                }

                // Handle remainder
                for i in 0..remainder {
                    let idx = chunks * 4 + i;
                    let tile_x = tile_x_start + idx as u8;
                    let color_idx = bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER) as usize;
                    let global_idx = sub_palette[color_idx].0 as usize;
                    let color = palette[global_idx];

                    let final_color = if color.a() > 0 {
                        let z_value = if is_fg { Z_BG_FOREGROUND } else { Z_BG_TILE };
                        color.with_z(z_value)
                    } else {
                        bg_color.with_z(Z_BG_COLOR)
                    };

                    *dst_ptr.add(idx) = final_color;
                }
            }

            x += pixels_to_process;
        }
    }

    #[inline(always)]
    fn pre_render_line(&mut self) {
        let width = self.vid.width().min(512);

        // Render both buffers independently (I hope CPU parallelism kicks in!)
        self.pre_render_background(width);
        self.pre_render_sprites(width);

        // Reset x position for iteration
        self.x = 0;
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = (RGBA32, Coords);

    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y > self.vid.max_y() as u16 {
            return None;
        }

        // Composite sprite and background buffers
        let sprite = self.sprite_buffer[self.x as usize];
        let bg = self.bg_buffer[self.x as usize];

        // Check viewport in y
        // let in_viewport = self.y >= self.vid.view_top && self.y <= self.vid.view_bottom;
        // // Simple alpha check - sprite wins if it has any alpha
        // let final_color = if in_viewport {
        //     if sprite.a() > 0 { sprite } else { bg }
        // } else {
        //     self.bg_color
        // };

        // Z-buffer based compositing - higher z value wins
        let in_viewport = self.y >= self.vid.view_top && self.y <= self.vid.view_bottom;
        let final_color = if in_viewport {
            // Compare z values: sprite has alpha and higher/equal z wins
            if sprite.a() > 0 && sprite.z() >= bg.z() {
                sprite
            } else if bg.a() > 0 {
                bg
            } else {
                self.bg_color.with_z(Z_BG_COLOR)
            }
        } else {
            self.bg_color.with_z(Z_BG_COLOR)
        };

        // results
        let color = RGBA32::from(final_color);
        let coords = Coords { x: self.x as i16, y: self.y as i16 };

        // Increment screen position
        self.x += 1;

        // Check if we need to go to the next line
        if self.x == self.vid.width() {
            self.x = 0;
            self.y += 1;

            // Prepare next line if not at end
            if self.y <= self.vid.max_y() as u16 {
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
