use crate::*;

/// This trait allows references to BGMaps of different sizes to be used by the Pixel Iterator.
pub trait DynamicBGMap: core::fmt::Debug {
    fn cells(&self) -> &[Cell];
    fn cells_mut(&mut self) -> &mut [Cell];
    fn columns(&self) -> u16;
    fn rows(&self) -> u16;
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn len(&self) -> usize;
    fn set_cell(&mut self, op: BgOp);
    fn set_id(&mut self, col: u16, row: u16, tile_id: TileID);
    fn set_flags(&mut self, col: u16, row: u16, flags: TileFlags);
    fn get_id(&self, col: u16, row: u16) -> Option<TileID>;
    fn get_flags(&self, col: u16, row: u16) -> Option<TileFlags>;
}

/// A simple collection of Cells, split into a number of columns and rows that can never exceed the
/// constant CELL_COUNT capacity.
#[derive(Debug, Clone)]
pub struct BGMap<const CELL_COUNT: usize> {
    pub cells: [Cell; CELL_COUNT],
    pub columns: u16,
    pub rows: u16,
}

/// A simple packet of required cells to fully set the attributes on a BG tile.
#[derive(Debug, Clone, Copy)]
pub struct BgOp {
    pub col: u16,
    pub row: u16,
    pub tile_id: TileID,
    pub flags: TileFlags,
}

impl<const CELL_COUNT: usize> BGMap<CELL_COUNT> {
    pub fn new(columns: u16, rows: u16) -> Self {
        assert!(
            CELL_COUNT < u16::MAX as usize,
            err!("Invalid BGMap CELL_COUNT (maximum number of tiles), must be lower than {}"),
            u16::MAX
        );
        assert!(
            (columns as usize * rows as usize) <= CELL_COUNT,
            err!("Invalid tilemap dimensions {} x {} = {} (exceeds CELL_COUNT value of {})"),
            columns,
            rows,
            columns as u32 * rows as u32,
            CELL_COUNT,
        );
        assert!(
            columns > 0 && rows > 0,
            err!("BGMap dimensions can't be zero")
        );
        Self {
            cells: core::array::from_fn(|_| Cell::default()),
            columns,
            rows,
        }
    }

    pub fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= CELL_COUNT,
            err!("Invalid column count")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("BGMap dimensions can't be zero")
        );
        self.columns = columns;
        self.rows = rows;
    }

    /// Returns None if coords are out of map. not sure if useful yet
    /// since the Pixel Iterator uses its own, wrapping coordinates.
    pub fn get_index(&self, col: u16, row: u16) -> Option<usize> {
        if col as usize >= self.columns as usize || row as usize >= self.rows as usize {
            return None;
        }
        Some((row as usize * self.columns as usize) + col as usize)
    }
}

impl<const CELL_COUNT: usize> DynamicBGMap for BGMap<CELL_COUNT> {
    fn cells(&self) -> &[Cell] {
        &self.cells
    }

    fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    fn columns(&self) -> u16 {
        self.columns
    }

    fn rows(&self) -> u16 {
        self.rows
    }

    fn width(&self) -> u16 {
        self.columns as u16 * TILE_SIZE as u16
    }

    fn height(&self) -> u16 {
        self.rows as u16 * TILE_SIZE as u16
    }

    fn len(&self) -> usize {
        CELL_COUNT
    }

    fn set_cell(&mut self, op: BgOp) {
        if let Some(index) = self.get_index(op.col, op.row) {
            self.cells[index].id = op.tile_id;
            self.cells[index].flags = op.flags;
        }
    }

    fn set_id(&mut self, col: u16, row: u16, tile_id: TileID) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].id = tile_id;
        }
    }

    fn set_flags(&mut self, col: u16, row: u16, flags: TileFlags) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].flags = flags;
        }
    }

    fn get_id(&self, col: u16, row: u16) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].id)
    }

    fn get_flags(&self, col: u16, row: u16) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].flags)
    }
}
