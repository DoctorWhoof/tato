use crate::*;
use core::array::from_fn;

type PaletteRemap = [u8; COLORS_PER_PALETTE as usize];

/// A "Memory Bank" that contains the actual tile pixels, a color palette
/// and color mappings for tile reuse with different colors.
/// Can be used both as:
/// - Const data (generated at build time by BankBuilder)
/// - Runtime mutable data (with tracking fields)
#[derive(Debug, Clone)]
pub struct Bank {
    pub tiles: [Tile<4>; TILE_COUNT],
    pub palette: [RGBA12; COLORS_PER_PALETTE as usize],
    pub color_mapping: [[u8; COLORS_PER_PALETTE as usize]; COLOR_MAPPING_COUNT as usize],
    // Runtime tracking (set to appropriate values for const banks, 0 for runtime banks)
    pub tile_head: u8,
    pub palette_head: u8,
    pub color_mapping_head: u8,
}

impl Bank {
    pub const fn new() -> Self {
        Self {
            tiles: [Tile::new(0, 0, 0, 0); TILE_COUNT],
            palette: PALETTE_DEFAULT,
            color_mapping: [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
                COLOR_MAPPING_COUNT as usize],
            tile_head: 0,
            palette_head: 0,
            color_mapping_head: 1, // 0 is always identity mapping
        }
    }

    pub fn reset(&mut self) {
        // Simply sets internal counters to 0.
        self.tile_head = 0;
        // Will reset colors to their defaults
        self.reset_palettes();
        // Reset color mappings to identity
        self.reset_color_mappings();
    }

    pub fn reset_palettes(&mut self) {
        self.palette = PALETTE_DEFAULT;
        self.palette_head = 0;
    }

    pub fn reset_color_mappings(&mut self) {
        // Reset all mappings to identity
        self.color_mapping = from_fn(|_| from_fn(|i| i as u8));
        self.color_mapping_head = 1; // 0 is always identity
    }

    pub fn push_color(&mut self, color: RGBA12) -> ColorID {
        assert!(self.palette_head < COLORS_PER_PALETTE as u8, "Palette capacity reached");
        let id = ColorID(self.palette_head);
        self.palette[self.palette_head as usize] = color;
        self.palette_head += 1;
        id
    }

    pub fn load_default_colors(&mut self) {
        self.palette = [
            RGBA12::TRANSPARENT, // 0
            RGBA12::BLACK,       // 1
            RGBA12::GRAY,        // 2
            RGBA12::WHITE,       // 3
            RGBA12::DARK_RED,    // 4
            RGBA12::RED,         // 5
            RGBA12::LIGHT_RED,   // 6
            RGBA12::ORANGE,      // 7
            RGBA12::YELLOW,      // 8
            RGBA12::DARK_GREEN,  // 9
            RGBA12::GREEN,       // 10
            RGBA12::LIGHT_GREEN, // 11
            RGBA12::DARK_BLUE,   // 12
            RGBA12::BLUE,        // 13
            RGBA12::LIGHT_BLUE,  // 14
            RGBA12::PINK,        // 15
        ];
        self.palette_head = 16;
    }

    pub fn set_color(&mut self, id: ColorID, color: RGBA12) {
        assert!(id.0 < COLORS_PER_PALETTE as u8, "Invalid color ID");
        self.palette[id.0 as usize] = color;
    }

    pub fn palette_cycle(&mut self, color_mapping: u8, start_index: u8, end_index: u8, delta: i8) {
        let original_colors = self.color_mapping[color_mapping as usize];

        for index in start_index as isize..=end_index as isize {
            let mut new_index = index + delta as isize;
            if delta > 0 {
                if new_index > end_index as isize {
                    new_index = start_index as isize;
                }
            } else {
                if new_index < start_index as isize {
                    new_index = end_index as isize;
                }
            }
            let color = &mut self.color_mapping[color_mapping as usize][index as usize];
            *color = original_colors[new_index as usize];
        }
    }

    pub fn tile_count(&self) -> usize {
        self.tile_head as usize
    }

    pub fn color_count(&self) -> u8 {
        self.palette_head
    }

    pub fn tile_capacity(&self) -> usize {
        TILE_COUNT
    }

    /// Restore tile counter to a previous state (for checkpoint/restore)
    /// Warning: Caller must ensure this is a valid previous state!
    pub fn restore_tile_count(&mut self, count: u8) {
        assert!(count as usize <= TILE_COUNT, "Invalid tile count");
        self.tile_head = count;
    }

    /// Restore palette counters to previous state (for checkpoint/restore)
    /// Warning: Caller must ensure these are valid previous states!
    pub fn restore_palette_state(&mut self, color_count: u8, color_mapping_count: u8) {
        assert!(color_count <= COLORS_PER_PALETTE as u8, "Invalid color count");
        assert!(color_mapping_count <= 16, "Invalid color mapping count");
        self.palette_head = color_count;
        self.color_mapping_head = color_mapping_count;
    }

    /// Adds a single tile, returns a TileID
    pub fn add_tile(&mut self, tile: &Tile<4>) -> TileID {
        assert!((self.tile_head as usize) < TILE_COUNT, err!("Tileset capacity reached"));
        let result = TileID(self.tile_head);
        // Copy tile data to bank
        let dest_index = self.tile_head as usize;
        self.tiles[dest_index] = *tile;
        self.tile_head += 1;
        result
    }

    /// Push a single color mapping to the bank
    /// Returns the index where the mapping was stored
    pub fn push_color_mapping(&mut self, mapping: [u8; COLORS_PER_PALETTE as usize]) -> u8 {
        assert!(self.color_mapping_head < 16, "Color mapping capacity reached");

        // Check if this is the identity mapping
        let is_identity = mapping.iter().enumerate().all(|(i, &v)| i == v as usize);
        if is_identity {
            return 0; // Always use index 0 for identity
        }

        // Check if this mapping already exists
        for i in 1..self.color_mapping_head {
            if self.color_mapping[i as usize] == mapping {
                return i; // Reuse existing mapping
            }
        }

        // Add new mapping
        let index = self.color_mapping_head;
        self.color_mapping[index as usize] = mapping;
        self.color_mapping_head += 1;
        index
    }

    /// Get the current count of color mappings
    pub fn color_mapping_count(&self) -> u8 {
        self.color_mapping_head
    }

    /// Adds unique colors to the bank, and returns a palette remap, if any
    pub fn append_colors(&mut self, colors: &[RGBA12]) -> Result<PaletteRemap, &'static str> {
        let mut color_remap = [0u8; COLORS_PER_PALETTE as usize];

        for src_color_idx in 0..colors.len() {
            let src_color = colors[src_color_idx];

            // Check if this color already exists in dest palette
            let mut found_idx = None;
            for dest_color_idx in 0..self.palette_head as usize {
                if self.palette[dest_color_idx] == src_color {
                    found_idx = Some(dest_color_idx);
                    break;
                }
            }

            if let Some(existing_idx) = found_idx {
                // Color already exists, reuse it
                color_remap[src_color_idx] = existing_idx as u8;
            } else {
                // New color, add it
                if self.palette_head >= COLORS_PER_PALETTE as u8 {
                    return Err("Not enough space in bank for colors");
                }
                color_remap[src_color_idx] = self.palette_head;
                self.palette[self.palette_head as usize] = src_color;
                self.palette_head += 1;
            }
        }
        Ok(color_remap)
    }

    pub fn append_tiles(
        &mut self,
        source: &Bank,
        color_remap: Option<PaletteRemap>,
    ) -> Result<u8, &'static str> {
        let tile_offset = self.tile_head;
        let source_tile_count = source.tile_head;
        let remap = match color_remap {
            Some(remap) => remap,
            None => core::array::from_fn(|i| i as u8),
        };

        // Check if we have space for tiles
        if (self.tile_head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }

        // Copy tiles while remapping pixel indices
        for tile_idx in 0..source_tile_count as usize {
            let mut remapped_tile = source.tiles[tile_idx];

            // Remap each pixel in the tile
            for cluster_idx in 0..remapped_tile.clusters.len() {
                for pixel_idx in 0..8 {
                    let old_color_idx = remapped_tile.clusters[cluster_idx].get_subpixel(pixel_idx);
                    let new_color_idx = remap[old_color_idx as usize];
                    remapped_tile.clusters[cluster_idx].set_subpixel(new_color_idx, pixel_idx);
                }
            }

            self.tiles[self.tile_head as usize + tile_idx] = remapped_tile;
        }
        self.tile_head += source_tile_count;

        // Copy color mappings while remapping their color indices
        for i in 1..source.color_mapping_head as usize {
            // Remap the color mapping using our color_remap table
            let mut remapped_mapping = [0u8; COLORS_PER_PALETTE as usize];
            for j in 0..COLORS_PER_PALETTE as usize {
                let src_color_ref = source.color_mapping[i][j];
                if (src_color_ref as usize) < source.palette_head as usize {
                    remapped_mapping[j] = remap[src_color_ref as usize];
                } else {
                    remapped_mapping[j] = j as u8; // Identity for unused entries
                }
            }

            // Check if this remapped mapping already exists
            let mut exists = false;
            for j in 0..self.color_mapping_head as usize {
                if self.color_mapping[j] == remapped_mapping {
                    exists = true;
                    break;
                }
            }

            if !exists && color_remap.is_some() {
                println!("Appending color map {}: {:?}", self.color_mapping_head, color_remap);
                if self.color_mapping_head >= COLOR_MAPPING_COUNT as u8 {
                    return Err("Not enough space in bank for color mappings");
                }
                self.color_mapping[self.color_mapping_head as usize] = remapped_mapping;
                self.color_mapping_head += 1;
            }
        }

        Ok(tile_offset)
    }

    /// Appends another bank's data into this bank, useful for combining multiple const Banks.
    /// Returns the tile offset where the source bank's tiles start in this bank.
    pub fn append(&mut self, source: &Bank) -> Result<u8, &'static str> {
        // Check if we have space for tiles
        let source_tile_count = source.tile_head;
        if (self.tile_head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }
        let src_len = source.palette_head as usize;
        let color_remap = self.append_colors(&source.palette[..src_len])?;
        self.append_tiles(source, Some(color_remap))
    }
}

impl Default for Bank {
    fn default() -> Self {
        Self::new()
    }
}
