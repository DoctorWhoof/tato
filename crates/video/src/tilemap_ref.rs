use crate::*;

/// A read-only view of a tilemap that erases the CELL_COUNT const generic.
#[derive(Debug, Clone, Copy)]
pub struct TilemapRef<'a> {
    /// Reference to the cell data.
    pub cells: &'a [Cell],
    /// The number of columns.
    pub columns: u16,
    /// The number of rows.
    pub rows: u16,
}

/// Converts a Tilemap reference to a TilemapRef.
impl<'a, const CELL_COUNT: usize> From<&'a Tilemap<CELL_COUNT>> for TilemapRef<'a> {
    fn from(tilemap: &'a Tilemap<CELL_COUNT>) -> Self {
        Self {
            cells: &tilemap.cells()[..((tilemap.columns() as usize) * (tilemap.rows() as usize))],
            columns: tilemap.columns(),
            rows: tilemap.rows(),
        }
    }
}

impl<'a> TilemapRef<'a> {
    /// Creates a TilemapRef from a static Tilemap in a const context.
    pub const fn from_const_tilemap<const CELL_COUNT: usize>(tilemap: &'static Tilemap<CELL_COUNT>) -> Self {
        Self {
            cells: &tilemap.cells,
            columns: tilemap.columns,
            rows: tilemap.rows,
        }
    }

}

impl<'a> DynTilemap for TilemapRef<'a> {
    fn cells(&self) -> &[Cell] {
        self.cells
    }

    fn columns(&self) -> u16 {
        self.columns
    }

    fn rows(&self) -> u16 {
        self.rows
    }

    fn width(&self) -> i16 {
        self.columns as i16 * TILE_SIZE as i16
    }

    fn height(&self) -> i16 {
        self.rows as i16 * TILE_SIZE as i16
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    #[inline(always)]
    fn get_index(&self, col: i16, row: i16) -> Option<usize> {
        if col < 0 || row < 0 {
            return None;
        }
        if col as usize >= self.columns as usize || row as usize >= self.rows as usize {
            return None;
        }
        Some((row as usize * self.columns as usize) + col as usize)
    }

    #[inline(always)]
    fn get_coords(&self, index: usize) -> Option<(u16, u16)> {
        if index >= (self.columns as usize * self.rows as usize) {
            return None;
        }
        let col = (index % self.columns as usize) as u16;
        let row = (index / self.columns as usize) as u16;
        Some((col, row))
    }

    fn get_cell(&self, col: i16, row: i16) -> Option<Cell> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index])
    }

    fn get_id(&self, col: i16, row: i16) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].id)
    }

    fn get_flags(&self, col: i16, row: i16) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index].flags)
    }
}
