use tato_video::*;

#[derive(Debug)]
pub struct TilemapRef<'a> {
    pub cells: &'a [Cell],
    pub columns: u16,
    pub rows: u16,
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

    fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= self.cells().len(),
            err!("Invalid column count")
        );
        assert!(columns > 0 && rows > 0, err!("Tilemap dimensions can't be zero"));
        self.columns = columns;
        self.rows = rows;
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
