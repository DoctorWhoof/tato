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

    /// Appends just the tiles from an array. Colors are not processed.
    pub fn append_tile(&mut self, tile: &Tile<2>) -> Result<TileID, &'static str> {
        let id = self.tiles.add(tile);
        Ok(id)
    }

    /// Appends just the tiles from an array. Colors are not processed.
    pub fn append_tiles(&mut self, source: &[Tile<2>]) -> Result<u8, &'static str> {
        let tile_offset = u8::try_from(self.tiles.head) //
            .expect("Bank: Error, tile count is invalid");

        let source_tile_count =
            u8::try_from(source.len()) //
                .expect("Bank: Error, source length must be 256 or less ");

        // Check if we have space for tiles
        if (self.tiles.head as usize + source_tile_count as usize) > TILE_COUNT {
            return Err("Not enough space in bank for tiles");
        }

        // Copy tiles
        for tile in source.iter() {
            self.tiles.add(tile);
        }

        Ok(tile_offset)
    }

    /// Appends the tiles from a different bank without modifying them.
    pub fn append_tiles_from_bank(
        &mut self,
        source: &Bank,
        _color_remap: Option<PaletteRemap>,
    ) -> Result<u8, &'static str> {
        let source_tile_count = source.tiles.head as usize;
        self.append_tiles(&source.tiles.tiles[..source_tile_count])
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
