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

    // Pre-rendered complete line buffer (BG + sprites + bg_color)
    line_buffer: [RGBA12; 512], // Max supported line width
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
            scroll_x: vid.scroll_x,
            scroll_y: vid.scroll_y,
            bg_color: vid.bg_color,
            scanline: vid.sprite_gen.scanlines[0].clone(),
            line_buffer: [RGBA12::BG; 512],
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

    #[inline]
    fn call_line_irq(&mut self) {
        if let Some(func) = self.irq_y {
            let bg_map = self.bg_banks[self.bg_map_bank as usize];
            func(self, self.vid, bg_map);
        }
    }

    #[inline(always)]
    fn pre_render_line(&mut self) {
        let width = self.vid.width().min(512) as usize;
        let bg = self.bg_banks[self.bg_map_bank as usize];
        let line_y = self.y as i16;
        // screen width divided into 16 slots
        let slot_width = self.vid.width() as f32 / SLOTS_PER_LINE as f32;

        // Pre-calculate viewport bounds
        let view_left = self.vid.view_left as usize;
        let view_right = self.vid.view_right as usize;
        let view_top = self.vid.view_top as u16;
        let view_bottom = self.vid.view_bottom as u16;

        // Cache tile data to avoid redundant lookups
        let mut cached_bg_col = u16::MAX;
        let mut cached_bg_flags = TileFlags::default();
        let mut cached_bg_cluster = Cluster::<2>::default();

        // Single pass render
        for x in 0..width {
            // Check viewport first
            let in_viewport = x >= view_left
                && self.y >= view_top
                && x <= view_right
                && self.y <= view_bottom;

            if !in_viewport {
                self.line_buffer[x] = self.bg_color;
                continue;
            }

            // Check for sprites at this position
            let mut sprite_color = RGBA12::BG;
            if self.scanline.mask > 0 {
                let slot = (x as f32 / slot_width) as u16;
                if self.scanline.mask & (1 << slot) != 0 {
                    // Check sprites in reverse order (front to back)
                    for n in (0..self.scanline.sprite_count as usize).rev() {
                        let sprite_id = self.scanline.sprites[n] as usize;
                        let sprite = &self.vid.sprite_gen.sprites[sprite_id];

                        let sprite_x = x as i16 - sprite.x;
                        if sprite_x < 0 || sprite_x >= TILE_SIZE as i16 {
                            continue;
                        }

                        let sprite_y = line_y - sprite.y;
                        if sprite_y < 0 || sprite_y >= TILE_SIZE as i16 {
                            continue;
                        }

                        // Inline transform for better performance
                        let (tx, ty) = if sprite.flags.is_rotated() {
                            let rotated_x = TILE_SIZE as i16 - 1 - sprite_y;
                            let rotated_y = sprite_x;
                            if sprite.flags.is_flipped_x() {
                                (rotated_x, TILE_SIZE as i16 - 1 - rotated_y)
                            } else if sprite.flags.is_flipped_y() {
                                (TILE_SIZE as i16 - 1 - rotated_x, rotated_y)
                            } else {
                                (rotated_x, rotated_y)
                            }
                        } else {
                            let tx = if sprite.flags.is_flipped_x() { TILE_SIZE as i16 - 1 - sprite_x } else { sprite_x };
                            let ty = if sprite.flags.is_flipped_y() { TILE_SIZE as i16 - 1 - sprite_y } else { sprite_y };
                            (tx, ty)
                        };

                        let bank = self.tile_banks[self.fg_tile_bank as usize];
                        let tile = &bank.tiles[sprite.id.0 as usize];
                        let pixel = tile.get_pixel(tx as u8, ty as u8) as usize;
                        let palette = sprite.flags.palette().id();
                        let index = bank.sub_palettes[palette][pixel].0;
                        let color = bank.palette[index as usize];

                        if color.a() > 0 {
                            sprite_color = color;
                            break;
                        }
                    }
                }
            }

            // If we found a sprite, use it
            if sprite_color.a() > 0 {
                self.line_buffer[x] = sprite_color;
                continue;
            }

            // Otherwise, render background
            let bg_x = (x as i16 + self.scroll_x).rem_euclid(bg.width() as i16) as u16;
            let bg_y = (line_y + self.scroll_y).rem_euclid(bg.height() as i16) as u16;

            // Check if outside BG map bounds
            let raw_x = x as i16 + self.scroll_x;
            let raw_y = line_y + self.scroll_y;
            let outside_bg = !self.vid.wrap_bg &&
                (raw_x < 0 || raw_y < 0 || raw_x >= bg.width() as i16 || raw_y >= bg.height() as i16);

            if outside_bg {
                self.line_buffer[x] = self.bg_color;
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

                // Calculate local tile Y coordinate (same for all pixels in this line)
                let tile_y = (bg_y % TILE_SIZE as u16) as u8;

                // Get the tile cluster
                let bank = self.tile_banks[self.bg_tile_bank as usize];
                let tile_clusters = &bank.tiles[bg_tile_id as usize].clusters;
                cached_bg_cluster = Cluster::from_tile(tile_clusters, cached_bg_flags, tile_y, TILE_SIZE);
            }

            // Calculate local tile X coordinate
            let tile_x = (bg_x % TILE_SIZE as u16) as u8;

            // Get background color
            let bank = self.tile_banks[self.bg_tile_bank as usize];
            let color_idx = cached_bg_cluster.get_subpixel(tile_x % PIXELS_PER_CLUSTER);
            let palette_idx = cached_bg_flags.palette().0 as usize;
            let global_idx = bank.sub_palettes[palette_idx][color_idx as usize].0 as usize;
            let color = bank.palette[global_idx];

            // Check if this is a foreground tile that should override sprites
            if cached_bg_flags.is_fg() && color.a() > 0 {
                self.line_buffer[x] = color;
            } else {
                // Use color if opaque, otherwise background
                self.line_buffer[x] = if color.a() > 0 { color } else { self.bg_color };
            }
        }

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

        // Simply read from pre-rendered buffer
        let color = RGBA32::from(self.line_buffer[self.x as usize]);
        let result_coords = ScreenCoords {
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
        Some((color, result_coords))
    }
}
