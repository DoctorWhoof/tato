use crate::*;

const BGMAP_LEN: usize = BG_COLUMNS as usize * BG_ROWS as usize;

/// A hardcoded "Tilemap". Its dimensions are larger than the screen dimensions, allowing
/// scrolling in both axis. For endless scrolling simply update the off-screen tiles before
/// you reach them, or turn on _*Videochip.wrap_bg*_ to repeat the current contents beyond its edges.
#[derive(Debug, Clone)]
pub struct BGMap {
    pub tiles: [TileID; BGMAP_LEN],
    pub flags: [TileFlags; BGMAP_LEN],
    /// The effective width in pixels, must be smaller than BG_WIDTH.
    /// Although the maximum number of tiles in a BG Map is determined by BG_WIDTH and BG_HEIGHT,
    /// you can use a smaller number if you wish. This is useful when "wrapping" the BG, for instance.
    /// Attempting to set this higher than BG_WIDTH will panic.
    pub width: u16,
    /// The effective height in pixels, must be smaller than BG_HEIGHT.
    pub height: u16,
}

/// A simple packet of required data to fully set the attributes on a BG tile.
#[derive(Debug, Clone, Copy)]
pub struct BgBundle {
    pub col: u16,
    pub row: u16,
    pub tile_id: TileID,
    pub flags: TileFlags,
}

impl BGMap {
    pub fn new(width: u16, height: u16) -> Self {
        assert!(width <= BG_WIDTH, "BG Map: width can't exceed {}", BG_WIDTH);
        assert!(height <= BG_HEIGHT, "BG Map: height can't exceed {}", BG_HEIGHT);
        Self {
            tiles: core::array::from_fn(|_| TileID(0)),
            flags: core::array::from_fn(|_| TileFlags::default()),
            width,
            height,
        }
    }

    // Returns None if coords are out of map. not sure if useful yet.
    // Iterator uses its own, wrapping coordinates.
    fn get_index(&self, col: u16, row: u16) -> Option<usize> {
        #[cfg(debug_assertions)]
        {
            if col as usize >= BG_COLUMNS as usize || row as usize >= BG_ROWS as usize {
                return None;
            }
        }
        Some((row as usize * BG_COLUMNS as usize) + col as usize)
    }

    pub fn set_tile(&mut self, data: BgBundle) {
        if let Some(index) = self.get_index(data.col, data.row) {
            self.tiles[index] = data.tile_id;
            self.flags[index] = data.flags;
        }
    }

    pub fn set_id(&mut self, col: u16, row: u16, tile_id: impl Into<TileID>) {
        if let Some(index) = self.get_index(col, row) {
            self.tiles[index] = tile_id.into();
        }
    }

    pub fn set_flags(&mut self, col: u16, row: u16, flags: impl Into<TileFlags>) {
        if let Some(index) = self.get_index(col, row) {
            self.flags[index] = flags.into();
        }
    }

    pub fn get_id(&self, col: u16, row: u16) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.tiles[index])
    }

    pub fn get_flags(&self, col: u16, row: u16) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.flags[index])
    }
}
