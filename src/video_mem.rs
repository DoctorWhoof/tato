use crate::*;

#[derive(Debug, Clone)]
pub struct VideoMemory<const TILES:usize, const CELLS:usize> {
    pub tiles: [Tile<2>; TILES],
    pub bg: BGMap<CELLS>,
    // Everything that needs to be counted
    tile_head: u16,
}

impl<const TILES:usize, const CELLS:usize> VideoMemory<TILES, CELLS> {
    pub fn new() -> Self {
        Self {
            tiles: core::array::from_fn(|_| Tile::default()),
            // "Flat" BgOp data for maps and anims
            bg: BGMap::new(32, 32),
            tile_head: 0,
        }
    }

    /// Does not erase the contents! Simply sets its internal count to 0.
    pub fn reset(&mut self) {
        self.tile_head = 0;
    }

    pub fn tile_count(&self) -> usize {
        self.tile_head as usize
    }

    pub fn tile_capacity(&self) -> usize {
        TILES
    }

    pub fn cell_capacity(&self) -> usize {
        CELLS
    }

    /// Adds a single tile, returns a TileID
    pub fn add_tile(&mut self, tile: &Tile<2>) -> TileID {
        assert!((self.tile_head as usize) < TILES, err!("Tileset capacity reached"));
        let result = TileID(self.tile_head);
        // Copy tile data to bank
        let dest_index = self.tile_head as usize;
        self.tiles[dest_index] = tile.clone();
        self.tile_head += 1;
        result
    }

    /// Get a specific tile within a tileset
    pub fn get_tile(&self, index: u16) -> Option<&Tile<2>> {
        if index < self.tile_head {
            Some(&self.tiles[index as usize])
        } else {
            None
        }
    }
}
