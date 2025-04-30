use crate::*;

const BGMAP_LEN: usize = BG_MAX_COLUMNS as usize * BG_MAX_ROWS as usize;

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
    pub columns: u8,
    /// The effective height in pixels, must be smaller than BG_HEIGHT.
    pub rows: u8,
}

/// A simple packet of required data to fully set the attributes on a BG tile.
#[derive(Debug, Clone, Copy)]
pub struct BgBundle {
    pub col: u8,
    pub row: u8,
    pub tile_id: TileID,
    pub flags: TileFlags,
}

impl BGMap {
    pub fn new(columns: u8, rows: u8) -> Self {
        assert!(columns <= BG_MAX_COLUMNS, "BG Map: width can't exceed {}", BG_MAX_COLUMNS);
        assert!(rows <= BG_MAX_ROWS, "BG Map: height can't exceed {}", BG_MAX_ROWS);
        Self {
            tiles: core::array::from_fn(|_| TileID(0)),
            flags: core::array::from_fn(|_| TileFlags::default()),
            columns,
            rows,
        }
    }

    pub fn width(&self) -> u16 {
        self.columns as u16 * MIN_TILE_SIZE as u16
    }

    pub fn height(&self) -> u16 {
        self.rows as u16 * MIN_TILE_SIZE as u16
    }

    // Returns None if coords are out of map. not sure if useful yet.
    // Iterator uses its own, wrapping coordinates.
    fn get_index(&self, col: u8, row: u8) -> Option<usize> {
        #[cfg(debug_assertions)]
        {
            if col as usize >= self.columns as usize || row as usize >= self.rows as usize {
                return None;
            }
        }
        Some((row as usize * self.columns as usize) + col as usize)
    }

    pub fn set_tile(&mut self, data: BgBundle) {
        if let Some(index) = self.get_index(data.col, data.row) {
            self.tiles[index] = data.tile_id;
            self.flags[index] = data.flags;
        }
    }

    pub fn set_id(&mut self, col: u8, row: u8, tile_id: impl Into<TileID>) {
        if let Some(index) = self.get_index(col, row) {
            self.tiles[index] = tile_id.into();
        }
    }

    pub fn set_flags(&mut self, col: u8, row: u8, flags: impl Into<TileFlags>) {
        if let Some(index) = self.get_index(col, row) {
            self.flags[index] = flags.into();
        }
    }

    pub fn get_id(&self, col: u8, row: u8) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.tiles[index])
    }

    pub fn get_flags(&self, col: u8, row: u8) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.flags[index])
    }
}
