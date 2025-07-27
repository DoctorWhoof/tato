use crate::*;

/// A fast (no "dyn" required), read-only reference of a Tilemap with any size
/// that allows erasing the "CELL_COUNT" const generic from a Tilemap.
/// You can easily convert from a Tilemap to a TilemapRef using ".into()"
/// in any field that requires a TilemapRef.
#[derive(Debug, Clone, Copy)]
pub struct TilemapRef<'a> {
    pub cells: &'a [Cell],
    pub columns: u16,
    pub rows: u16,
}

// Conversion from Tilemap reference to TilemapRef
impl<'a, const CELL_COUNT: usize> From<&'a Tilemap<CELL_COUNT>> for TilemapRef<'a> {
    fn from(tilemap: &'a Tilemap<CELL_COUNT>) -> Self {
        Self {
            cells: &tilemap.cells[..((tilemap.columns as usize) * (tilemap.rows as usize))],
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

    fn width(&self) -> u16 {
        self.columns as u16 * TILE_SIZE as u16
    }

    fn height(&self) -> u16 {
        self.rows as u16 * TILE_SIZE as u16
    }

    fn len(&self) -> usize {
        self.cells.len()
    }

    #[inline(always)]
    fn get_index(&self, col: u16, row: u16) -> Option<usize> {
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

    fn get_cell(&self, col: u16, row: u16) -> Option<Cell> {
        let index = self.get_index(col, row)?;
        Some(self.cells[index])
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
