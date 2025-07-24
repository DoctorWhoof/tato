use tato_video::*;

#[derive(Debug)]
pub struct TilemapRef<'a> {
    pub cells: &'a mut [Cell],
    pub columns: u16,
    pub rows: u16,
}

impl<'a> DynTilemap for TilemapRef<'a> {
    fn cells(&self) -> &[Cell] {
        self.cells
    }

    fn cells_mut(&mut self) -> &mut [Cell] {
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
}
