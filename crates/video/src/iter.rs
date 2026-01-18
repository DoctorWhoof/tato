use core::array::from_fn;

use crate::*;

// Z-buffer priority constants for compositing
const Z_BG: u8 = 0; // Background color (lowest priority)
const Z_BG_TILE: u8 = 1; // Normal background tiles
const Z_SPRITE: u8 = 2; // Sprites
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
    pub tile_banks: [&'a Bank; BANK_COUNT],
    pub tilemaps: [TilemapRef<'a>; BG_BANK_COUNT],
    pub scroll_x: i16,
    pub scroll_y: i16,
    pub bg_color: RGBA12,   // Background color
    pub crop_color: RGBA12, // Crop color (outside viewport)

    // Dual buffers for parallel processing
    sprite_buffer: [RGBA12; MAX_RESOLUTION_X], // Sprite layer
    bg_buffer: [RGBA12; MAX_RESOLUTION_X],     // Background layer (tiles + bg_color)
}

impl<'a> PixelIter<'a> {
    pub fn new<T>(vid: &'a VideoChip, video_mem: &'a [Bank], bg_maps: &'a [&'a T]) -> Self
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        assert!(!video_mem.is_empty(), err!("Video Memory bank can't be empty"));
        assert!(
            video_mem.len() <= BANK_COUNT,
            err!("Video Memory bank count ({}) exceeds maximum ({})"),
            video_mem.len(),
            BANK_COUNT
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
            tile_banks: from_fn(|i| if i < video_mem.len() { &video_mem[i] } else { &video_mem[0] }),
            tilemaps: from_fn(|i| {
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
            crop_color: vid.crop_color,
            // scanline: vid.sprite_gen.scanlines[0].clone(),
            sprite_buffer: [RGBA12::TRANSPARENT.with_z(Z_SPRITE); MAX_RESOLUTION_X],
            bg_buffer: Self::generate_bg_color(0, vid),
        };

        // Pre-render first line (IRQ will be called inside pre_render_line)
        result.pre_render_line();
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
            let bg_map = self.tilemaps[self.bg_map_bank as usize];
            func(self, self.vid, &bg_map);
        }
    }

    #[inline]
    fn generate_bg_color(y: u16, vid: &VideoChip) -> [RGBA12; MAX_RESOLUTION_X] {
        if y < vid.view_top || y > vid.view_bottom {
            [vid.crop_color.with_z(Z_BG); MAX_RESOLUTION_X]
        } else {
            [RGBA12::TRANSPARENT.with_z(Z_BG); MAX_RESOLUTION_X]
        }
    }

    #[inline]
    fn pre_render_line(&mut self) {
        // Run Y IRQ before rendering the line
        self.call_line_irq();

        // Pre-calculate viewport bounds
        let width = self.vid.width().min(MAX_RESOLUTION_X as u16);
        let view_start = self.vid.view_left.max(0) as usize;
        let view_end = self.vid.view_right.min(width) as usize;
        if view_start >= view_end {
            return;
        }

        if self.y < self.vid.view_top {
            return;
        }

        if self.y > self.vid.view_bottom {
            // Clear bg bottom. This can probably be eliminated if I change how BG renders
            self.bg_buffer = Self::generate_bg_color(self.y, self.vid);
            self.sprite_buffer = [RGBA12::TRANSPARENT.with_z(Z_SPRITE); MAX_RESOLUTION_X];
            return;
        }

        // Render both buffers independently (I hope CPU parallelism kicks in!)
        // Clear sprite buffer only in viewport
        unsafe {
            let ptr = self.sprite_buffer.as_mut_ptr();
            for x in view_start..view_end {
                *ptr.add(x) = RGBA12::TRANSPARENT;
            }
        }
        self.pre_render_sprites(width, view_start, view_end);

        // Fast fill non-viewport areas with crop_color
        unsafe {
            // Fill start
            if view_start > 0 {
                let ptr = self.bg_buffer.as_mut_ptr();
                for i in 0..view_start {
                    *ptr.add(i) = self.crop_color.with_z(Z_BG);
                }
            }
            // Fill end
            if view_end < width as usize {
                let ptr = self.bg_buffer.as_mut_ptr();
                for i in view_end..width as usize {
                    *ptr.add(i) = self.crop_color.with_z(Z_BG);
                }
            }
        }
        self.pre_render_background(width);
    }

    #[inline]
    fn pre_render_sprites(&mut self, width: u16, view_start: usize, view_end: usize) {
        self.x = 0;
        // self.scanline = self.vid.sprite_gen.scanlines[self.y as usize].clone();
        let scanline = &self.vid.sprite_gen.scanlines[self.y as usize];

        // Early exit if no sprites or no viewport
        if scanline.mask == 0 {
            return;
        }

        let line_y = self.y as i16;
        let bank = self.tile_banks[self.fg_tile_bank as usize];

        // Process sprites from back to front
        for n in (0..scanline.sprite_count as usize).rev() {
            let sprite_id = scanline.sprites[n] as usize;
            let sprite = &self.vid.sprite_gen.sprites[sprite_id];

            if sprite.flags.is_invisible() {
                continue;
            }

            let sprite_y = line_y - sprite.y;
            if sprite_y < 0 || sprite_y >= TILE_SIZE as i16 {
                continue;
            }

            // Calculate sprite bounds clamped to viewport
            let sprite_start = sprite.x.max(0) as usize;
            let sprite_end = ((sprite.x + TILE_SIZE as i16).min(width as i16)) as usize;

            // Clamp to viewport
            let start_x = sprite_start.max(view_start);
            let end_x = sprite_end.min(view_end);

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

            if scanline.mask & slot_mask == 0 {
                continue;
            }

            // Pre-calculate transformation flags
            let flip_x = sprite.flags.is_flipped_x();
            let flip_y = sprite.flags.is_flipped_y();
            let rotated = sprite.flags.is_rotated();
            let tile = &bank.tiles.tiles[sprite.id.0 as usize];
            // let color_mapping = sprite.color_mapping;

            // Render sprite pixels - only in active slots!
            for x in start_x..end_x {
                // Check if this pixel is in an active slot
                let pixel_slot = (x as f32 / self.slot_width) as u16;
                if scanline.mask & (1 << pixel_slot) == 0 {
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
                let map_index = sprite.color_mapping as usize;
                let mapped_pixel = bank.colors.mapping[map_index][pixel] as usize;
                let color = bank.colors.palette[mapped_pixel];

                if color.a() > 0 {
                    self.sprite_buffer[x] = color.with_z(Z_SPRITE);
                }
            }
        }
    }

    #[inline]
    fn pre_render_background(&mut self, width: u16) {
        // Reset x position for iteration
        self.x = 0;
        let bg = self.tilemaps[self.bg_map_bank as usize];
        let line_y = self.y as i16;
        let bank = self.tile_banks[self.bg_tile_bank as usize];

        // Pre-calculate viewport bounds
        let view_start = self.vid.view_left.max(0) as usize;
        let view_end = self.vid.view_right.min(width) as usize;

        // Pre-calculate Y coordinates once
        let bg_y_base = line_y + self.scroll_y;
        let bg_height = bg.height() as i16;
        let bg_width = bg.width() as i16;

        // Check Y bounds once for entire line (when wrap_bg is false)
        if !self.wrap_bg && (bg_y_base < 0 || bg_y_base >= bg_height) {
            let bg_color = self.bg_color;
            for x in view_start..view_end {
                self.bg_buffer[x] = bg_color.with_z(Z_BG);
            }
            return;
        }

        let bg_y = bg_y_base.rem_euclid(bg_height) as u16;
        let bg_row = bg_y / TILE_SIZE as u16;
        let tile_y = (bg_y % TILE_SIZE as u16) as u8;
        let bg_columns = bg.columns() as usize;

        // Process viewport pixels in tile-aligned batches
        let mut x = view_start;

        while x < view_end {
            // Calculate starting BG X coordinate
            let bg_x_base = x as i16 + self.scroll_x;

            // Handle horizontal out of bounds
            if !self.wrap_bg {
                if bg_x_base < 0 {
                    // Skip negative pixels
                    let skip = (-bg_x_base).min((view_end - x) as i16) as usize;
                    let bg_color = self.bg_color;
                    for i in 0..skip {
                        self.bg_buffer[x + i] = bg_color.with_z(Z_BG);
                    }
                    x += skip;
                    continue;
                } else if bg_x_base >= bg_width {
                    // Fill rest with bg_color and exit
                    let bg_color = self.bg_color;
                    for i in x..view_end {
                        self.bg_buffer[i] = bg_color.with_z(Z_BG);
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
            let bg_tile_id = bg_cell.id.0 as usize;
            // let bg_color_mapping = bg_cell.color_mapping as usize;

            // Skip invisible tiles - fill with bg_color instead
            if bg_flags.is_invisible() {
                let tile_pixels_remaining = (TILE_SIZE as usize) - tile_x_start as usize;
                let viewport_pixels_remaining = view_end - x;
                let pixels_to_process = tile_pixels_remaining.min(viewport_pixels_remaining);

                let pixels_to_process = if !self.wrap_bg && bg_x_base >= 0 {
                    let max_x = (bg_width - bg_x_base) as usize;
                    pixels_to_process.min(max_x)
                } else {
                    pixels_to_process
                };

                let bg_color = self.bg_color;
                for i in 0..pixels_to_process {
                    self.bg_buffer[x + i] = bg_color.with_z(Z_BG);
                }
                x += pixels_to_process;
                continue;
            }

            // Get the tile cluster for this row
            let tile = &bank.tiles.tiles[bg_tile_id];
            let bg_cluster = Cluster::from_tile(&tile.clusters, bg_flags, tile_y, TILE_SIZE);

            // Pre-fetch palette data
            let palette = &bank.colors.palette;
            let is_fg = bg_flags.is_fg();
            let bg_color = self.bg_color;

            // Calculate pixels to process in this tile
            let tile_pixels_remaining = (TILE_SIZE as usize) - tile_x_start as usize;
            let viewport_pixels_remaining = view_end - x;
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

                let remap_id = bg_cell.color_mapping as usize;

                for chunk in 0..chunks {
                    let base_idx = chunk * 4;
                    for i in 0..4 {
                        let tile_x = tile_x_start + (base_idx + i) as u8;
                        let color_idx =
                            bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER) as usize;
                        let mapped_idx = bank.colors.mapping[remap_id][color_idx] as usize;
                        let color = palette[mapped_idx];

                        let final_color = if color.a() > 0 {
                            let z_value = if is_fg { Z_BG_FOREGROUND } else { Z_BG_TILE };
                            color.with_z(z_value)
                        } else {
                            bg_color.with_z(Z_BG)
                        };

                        *dst_ptr.add(base_idx + i) = final_color;
                    }
                }

                // Handle remainder
                for i in 0..remainder {
                    let idx = chunks * 4 + i;
                    let tile_x = tile_x_start + idx as u8;
                    let color_idx = bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER) as usize;
                    let mapped_idx = bank.colors.mapping[remap_id][color_idx] as usize;
                    let color = palette[mapped_idx];

                    let final_color = if color.a() > 0 {
                        let z_value = if is_fg { Z_BG_FOREGROUND } else { Z_BG_TILE };
                        color.with_z(z_value)
                    } else {
                        bg_color.with_z(Z_BG)
                    };

                    *dst_ptr.add(idx) = final_color;
                }
            }

            x += pixels_to_process;
        }
    }
}

impl<'a> Iterator for PixelIter<'a> {
    type Item = RGBA32;

    // Performance goal: iterator with no actual rendering.
    // Currently getting 0.2ms to 0.3ms in release
    fn next(&mut self) -> Option<Self::Item> {
        // End line reached
        if self.y > self.vid.max_y() as u16 {
            return None;
        }

        // Composite sprite and background buffers
        let sprite = self.sprite_buffer[self.x as usize];
        let bg = self.bg_buffer[self.x as usize];

        // results
        let color = RGBA32::from(
            // Compare z values: sprite has alpha and higher/equal z wins
            if sprite.a() > 0 && sprite.z() >= bg.z() {
                sprite
            } else if bg.a() > 0 {
                bg
            } else {
                self.bg_color.with_z(Z_BG)
            },
        );

        // Increment screen position
        self.x += 1;

        // Check if we need to go to the next line
        if self.x == self.vid.width() {
            self.x = 0;
            self.y += 1;
            // Pre-render the new line (IRQ will be called inside pre_render_line)
            self.pre_render_line();
        }

        // Return the pixel color
        Some(color)
    }
}
