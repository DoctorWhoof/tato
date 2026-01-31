use crate::*;

#[derive(Debug, Clone)]
pub struct TileBank {
    pub tiles: [Tile<2>; TILE_COUNT],
    pub(crate) head: u8,
}

impl TileBank {
    pub const fn new() -> Self {
        Self { tiles: [Tile::<2>::new(0, 0); TILE_COUNT], head: 0 }
    }

    pub const fn new_from(tiles: &[Tile<2>]) -> Self {
        // Create tiles array
        let mut tiles_array = [Tile::<2>::new(0, 0); TILE_COUNT];
        let mut i = 0;
        while i < TILE_COUNT {
            if i < tiles.len() {
                tiles_array[i] = tiles[i];
            }
            i += 1;
        }

        Self { tiles: tiles_array, head: tiles.len() as u8 }
    }

    pub fn reset(&mut self) {
        // Simply sets internal counters to 0. Existing tiles will remain.
        self.head = 0;
    }

    pub fn count(&self) -> usize {
        self.head as usize
    }

    pub fn capacity(&self) -> usize {
        256
    }

    /// Restore tile counter to a previous state (for checkpoint/restore)
    /// Warning: Caller must ensure this is a valid previous state!
    pub fn restore_state(&mut self, count: u8) {
        assert!(count as usize <= TILE_COUNT, "Invalid tile count");
        self.head = count;
    }

    /// Adds a single tile, returns a TileID
    pub fn add(&mut self, tile: &Tile<2>) -> TileID {
        assert!((self.head as usize) < TILE_COUNT, err!("Tileset capacity reached"));
        let result = TileID(self.head);
        // Copy tile data to bank
        let dest_index = self.head as usize;
        self.tiles[dest_index] = *tile;
        self.head += 1;
        result
    }
}
