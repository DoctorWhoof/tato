use crate::*;

#[derive(Debug, Clone)]
pub struct BGMap<const CELLS: usize> {
    pub cells: [Cell; CELLS],
    pub columns: u16,
    pub rows:u16,
}

/// A simple packet of required cells to fully set the attributes on a BG tile.
#[derive(Debug, Clone, Copy)]
pub struct BgOp {
    pub col: u16,
    pub row: u16,
    pub tile_id: TileID,
    pub flags: TileFlags,
}

impl<const CELLS: usize> BGMap<CELLS> {
    pub fn new(columns: u16, rows: u16) -> Self {
        assert!(
            CELLS < u16::MAX as usize,
            err!("Invalid BGMap CELLS (maximum number of tiles), must be lower than {}"),
            u16::MAX
        );
        assert!(
            (columns as usize * rows as usize) <= CELLS,
            err!("Invalid tilemap dimensions")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("BGMap dimensions can't be zero")
        );
        Self {
            cells: core::array::from_fn(|_| Cell::default()),
            columns,
            rows
        }
    }

    pub fn width(&self) -> u16 {
        self.columns as u16 * TILE_SIZE as u16
    }

    pub fn height(&self) -> u16 {
        self.rows as u16 * TILE_SIZE as u16
    }

    pub fn len(&self) -> usize {
        CELLS
    }

    pub fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= CELLS,
            err!("Invalid column count")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("BGMap dimensions can't be zero")
        );
        self.columns = columns;
        self.rows = rows;
    }

    // Returns None if coords are out of map. not sure if useful yet.
    // Iterator uses its own, wrapping coordinates.
    fn get_index(&self, col: u16, row: u16) -> Option<usize> {
        // #[cfg(debug_assertions)]
        // {
            if col as usize >= self.columns as usize || row as usize >= self.rows as usize {
                return None;
            }
        // }
        Some((row as usize * self.columns as usize) + col as usize)
    }

    pub fn set_cell(&mut self, op: BgOp) {
        if let Some(index) = self.get_index(op.col, op.row) {
            self.cells[index].id = op.tile_id;
            self.cells[index].flags = op.flags;
        }
    }

    pub fn set_id(&mut self, col: u16, row: u16, tile_id: impl Into<TileID>) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].id = tile_id.into();
        }
    }

    pub fn set_flags(&mut self, col: u16, row: u16, flags: impl Into<TileFlags>) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].flags = flags.into();
        }
    }

    pub fn get_id(&self, col: u16, row: u16) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].id)
    }

    pub fn get_flags(&self, col: u16, row: u16) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].flags)
    }
}
