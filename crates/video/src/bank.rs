use crate::*;

#[derive(Debug)]
pub struct Bank {
    pub tiles: TileBank,
    pub colors: ColorBank,
}

impl Bank {
    pub const fn new() -> Self {
        Self { tiles:TileBank::new(), colors:ColorBank::new() }
    }

    pub fn reset(&mut self) {
        self.tiles.reset();
        self.colors.reset_palettes();
        self.colors.reset_color_mappings();
    }

    /// Appends just the tiles from a different bank, remapping their colors
    /// if a color_remap is provided.
    pub fn append_tiles(
        &mut self,
        source: &Bank,
        color_remap: Option<PaletteRemap>,
    ) -> Result<u8, &'static str> {
        let tile_offset = self.tiles.head;
        let source_tile_count = source.tiles.head;
        let remap = match color_remap {
            Some(remap) => remap,
            None => core::array::from_fn(|i| i as u8),
        };

        // Check if we have space for tiles
        if (self.tiles.head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }

        // Copy tiles while remapping pixel indices
        for tile_idx in 0..source_tile_count as usize {
            let mut remapped_tile = source.tiles.tiles[tile_idx];

            // Remap each pixel in the tile
            for cluster_idx in 0..remapped_tile.clusters.len() {
                for pixel_idx in 0..8 {
                    let old_color_idx = remapped_tile.clusters[cluster_idx].get_subpixel(pixel_idx);
                    let new_color_idx = remap[old_color_idx as usize];
                    remapped_tile.clusters[cluster_idx].set_subpixel(new_color_idx, pixel_idx);
                }
            }

            self.tiles.tiles[self.tiles.head as usize + tile_idx] = remapped_tile;
        }
        self.tiles.head += source_tile_count;

        // Copy color mappings while remapping their color indices
        for i in 1..source.colors.mapping_head as usize {
            // Remap the color mapping using our color_remap table
            let mut remapped_mapping = [0u8; COLORS_PER_PALETTE as usize];
            for j in 0..COLORS_PER_PALETTE as usize {
                let src_color_ref = source.colors.mapping[i][j];
                if (src_color_ref as usize) < source.colors.palette_head as usize {
                    remapped_mapping[j] = remap[src_color_ref as usize];
                } else {
                    remapped_mapping[j] = j as u8; // Identity for unused entries
                }
            }

            // Check if this remapped mapping already exists
            let mut exists = false;
            for j in 0..self.colors.mapping_head as usize {
                if self.colors.mapping[j] == remapped_mapping {
                    exists = true;
                    break;
                }
            }

            if !exists && color_remap.is_some() {
                // println!("Appending color map {}: {:?}", self.colors.mapping_head, color_remap);
                if self.colors.mapping_head >= COLOR_MAPPING_COUNT as u8 {
                    return Err("Not enough space in bank for color mappings");
                }
                self.colors.mapping[self.colors.mapping_head as usize] = remapped_mapping;
                self.colors.mapping_head += 1;
            }
        }

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
        self.append_tiles(source, Some(color_remap))
    }
}
