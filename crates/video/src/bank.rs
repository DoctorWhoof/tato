use crate::*;

#[derive(Debug)]
pub struct Bank {
    pub tiles: TileBank,
    pub colors: ColorBank,
}

impl Bank {
    pub const fn new() -> Self {
        Self { tiles: TileBank::new(), colors: ColorBank::new() }
    }

    pub fn reset(&mut self) {
        self.tiles.reset();
        self.colors.reset_palettes();
    }

    /// Appends just the tiles from a tilebank. Colors are not processed.
    pub fn append_tiles(&mut self, source: &TileBank) -> Result<u8, &'static str> {
        let tile_offset = self.tiles.head;
        let source_tile_count = source.head;

        // Check if we have space for tiles
        if (self.tiles.head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }

        // Copy tiles while remapping pixel indices
        for tile_idx in 0..source_tile_count as usize {
            let tile = source.tiles[tile_idx];
            self.tiles.tiles[self.tiles.head as usize + tile_idx] = tile;
        }
        self.tiles.head += source_tile_count;
        Ok(tile_offset)
    }

    /// Appends the tiles from a different bank without modifying them.
    /// With TileColors per-cell, color mapping is handled by each Cell's colors field,
    /// not by modifying the tile data itself.
    pub fn append_tiles_from_bank(
        &mut self,
        source: &Bank,
        _color_remap: Option<PaletteRemap>,
    ) -> Result<u8, &'static str> {
        let tile_offset = self.tiles.head;
        let source_tile_count = source.tiles.head;

        // Check if we have space for tiles
        if (self.tiles.head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }

        // Copy tiles as-is without remapping pixel indices
        // The TileColors in each Cell will handle color mapping at render time
        for tile_idx in 0..source_tile_count as usize {
            let tile = source.tiles.tiles[tile_idx];
            self.tiles.tiles[self.tiles.head as usize + tile_idx] = tile;
        }
        self.tiles.head += source_tile_count;

        // Copy color mappings while remapping their color indices
        // Note: Both the INPUT indices (pixel values) and OUTPUT colors have been remapped.
        // Tile pixels that were originally 'j' are now stored as 'remap[j]'.
        // So when rendering reads 'remap[j]', we need mapping[remap[j]] to give the remapped output.

        // for i in 1..source.colors.mapping_head as usize {
        //     // Start with identity mapping
        //     let mut remapped_mapping: [u8; COLORS_PER_PALETTE as usize] =
        //         core::array::from_fn(|i| i as u8);

        //     for j in 0..COLORS_PER_PALETTE as usize {
        //         let src_color_ref = source.colors.mapping[i][j];
        //         let new_input_idx = remap[j] as usize;
        //         if (src_color_ref as usize) < source.colors.palette_head as usize {
        //             remapped_mapping[new_input_idx] = remap[src_color_ref as usize];
        //         } else {
        //             // For colors beyond the palette, use identity mapping based on j, not new_input_idx.
        //             // remap[j] may be 0 for j >= palette size, which would corrupt remapped_mapping[0].
        //             remapped_mapping[j] = j as u8;
        //         }
        //     }
        //     // Check if this remapped mapping already exists
        //     let mut exists = false;
        //     for j in 0..self.colors.mapping_head as usize {
        //         if self.colors.mapping[j] == remapped_mapping {
        //             exists = true;
        //             break;
        //         }
        //     }

        //     if !exists {
        //         if self.colors.mapping_head >= COLOR_MAPPING_COUNT as u8 {
        //             return Err("Not enough space in bank for color mappings");
        //         }
        //         self.colors.mapping[self.colors.mapping_head as usize] = remapped_mapping;
        //         self.colors.mapping_head += 1;
        //     }
        // }

        Ok(tile_offset)
    }

    /// Appends another bank's data into this bank, useful for combining multiple const Banks.
    /// Returns the tile offset where the source bank's tiles start in this bank.
    pub fn append(&mut self, source: &Bank) -> Result<u8, &'static str> {
        // Check if we have space for tiles
        let source_tile_count = source.tiles.head;
        if (self.tiles.head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }
        let src_len = source.colors.palette_head as usize;
        let color_remap = self.colors.append(&source.colors.palette[..src_len])?;
        self.append_tiles_from_bank(source, Some(color_remap))
    }
}
