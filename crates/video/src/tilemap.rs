use crate::*;
use tato_math::rect::Rect;

/// Trait for read-only tilemap operations, abstracting over different sizes.
pub trait DynTilemap: core::fmt::Debug {
    /// Slice of all cells in the tilemap.
    fn cells(&self) -> &[Cell];
    /// Number of cell columns.
    fn columns(&self) -> u16;
    /// Number of cell rows.
    fn rows(&self) -> u16;
    /// Width in pixels.
    fn width(&self) -> i16;
    /// Height in pixels.
    fn height(&self) -> i16;
    /// Total cell capacity of the tilemap.
    fn len(&self) -> usize;
    /// Converts column and row coordinates to a linear index.
    fn get_index(&self, col: i16, row: i16) -> Option<usize>;
    /// Converts a linear index to column and row coordinates.
    fn get_coords(&self, index: usize) -> Option<(u16, u16)>;
    /// Returns the cell at the given coordinates.
    fn get_cell(&self, col: i16, row: i16) -> Option<Cell>;
    /// Returns the tile ID at the given coordinates.
    fn get_id(&self, col: i16, row: i16) -> Option<TileID>;
    /// Returns the tile flags at the given coordinates.
    fn get_flags(&self, col: i16, row: i16) -> Option<TileFlags>;
}

/// A fixed-capacity grid of cells with a maximum size of CELL_COUNT.
#[derive(Debug, Clone)]
pub struct Tilemap<const CELL_COUNT: usize> {
    /// The cell storage array.
    pub cells: [Cell; CELL_COUNT],
    /// The number of active columns.
    pub columns: u16,
    /// The number of active rows.
    pub rows: u16,
}

/// An operation to set a cell at specific coordinates.
#[derive(Debug, Clone, Copy)]
pub struct BgOp {
    /// The target column.
    pub col: i16,
    /// The target row.
    pub row: i16,
    /// The cell data to set.
    pub cell: Cell,
}

impl<const CELL_COUNT: usize> Tilemap<CELL_COUNT> {
    /// Creates a new tilemap with the specified dimensions.
    ///
    /// Panics if dimensions are zero or exceed CELL_COUNT.
    pub fn new(columns: u16, rows: u16) -> Self {
        assert!(
            CELL_COUNT < u16::MAX as usize,
            err!("Invalid Tilemap CELL_COUNT (maximum number of tiles), must be lower than {}"),
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
        assert!(columns > 0 && rows > 0, err!("Tilemap dimensions can't be zero"));
        Self {
            cells: core::array::from_fn(|_| Cell::default()),
            columns,
            rows,
        }
    }

    /// Returns a read-only "view" of the tilemap, which can be passed around
    /// without knowledge of the CELL_COUNT const generic.
    pub fn as_ref(&self) -> TilemapRef<'_> {
        self.into()
    }

    /// Changes the active dimensions of the tilemap.
    ///
    /// Panics if dimensions are zero or exceed capacity.
    pub fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= self.cells().len(),
            err!("Invalid dimensions (exceeds capacity)")
        );
        assert!(columns > 0 && rows > 0, err!("Tilemap dimensions can't be zero"));
        self.columns = columns;
        self.rows = rows;
    }

    /// Sets the cell at the given coordinates. Does nothing if out of bounds.
    pub fn set_cell(&mut self, col: i16, row: i16, cell: Cell) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index] = cell
        }
    }

    /// Applies a BgOp operation. Does nothing if out of bounds.
    pub fn set_op(&mut self, op: BgOp) {
        if let Some(index) = self.get_index(op.col, op.row) {
            self.cells[index] = op.cell;
        }
    }

    /// Returns a mutable reference to the cell at the given coordinates.
    pub fn cell_mut(&mut self, col: i16, row: i16) -> Option<&mut Cell> {
        if let Some(index) = self.get_index(col, row) {
            Some(&mut self.cells[index]) //
        } else {
            None
        }
    }

    /// Sets the tile ID at the given coordinates. Does nothing if out of bounds.
    pub fn set_id(&mut self, col: i16, row: i16, tile_id: TileID) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].id = tile_id;
        }
    }

    /// Sets the tile flags at the given coordinates. Does nothing if out of bounds.
    pub fn set_flags(&mut self, col: i16, row: i16, flags: TileFlags) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].flags = flags;
        }
    }

    /// Sets the palette colors at the given coordinates. Does nothing if out of bounds.
    pub fn set_colors(&mut self, col: i16, row: i16, colors: Palette) {
        if let Some(index) = self.get_index(col, row) {
            self.cells[index].colors = colors;
        }
    }
    /// Copies a rectangular region from a cells array to this tilemap.
    ///
    /// If `src_rect` is None, copies the entire source. If `dst_rect` is None, pastes at (0,0).
    /// Negative destination coordinates are clipped automatically.
    pub fn copy_from_cells(
        &mut self,
        cells: &[Cell],
        columns: i16,
        rows: i16,
        src_rect: Option<Rect<i16>>,
        dst_rect: Option<Rect<i16>>,
        tile_offset: u8,
    ) {
        // Determine source rectangle
        let src_x = src_rect.map_or(0, |r| r.x);
        let src_y = src_rect.map_or(0, |r| r.y);
        let src_w = src_rect.map_or(columns, |r| r.w);
        let src_h = src_rect.map_or(rows, |r| r.h);

        // Make sure source rectangle is within bounds
        let src_w = i16::min(src_w, columns - src_x);
        let src_h = i16::min(src_h, rows - src_y);

        // Determine destination rectangle
        let dst_x = dst_rect.map_or(0, |r| r.x) as i16;
        let dst_y = dst_rect.map_or(0, |r| r.y) as i16;
        let dst_w = dst_rect.map_or(self.columns as i16, |r| r.w);
        let dst_h = dst_rect.map_or(self.rows as i16, |r| r.h);

        // Calculate clipping for negative coordinates
        let clip_x = if dst_x < 0 { -dst_x } else { 0 };
        let clip_y = if dst_y < 0 { -dst_y } else { 0 };

        // Adjust source and destination starting points
        let effective_dst_x = i16::max(0, dst_x);
        let effective_dst_y = i16::max(0, dst_y);

        // Calculate effective width and height after clipping
        let effective_width = i16::max(
            0,
            i16::min(
                i16::min(src_w - clip_x, dst_w - clip_x),
                self.columns as i16 - effective_dst_x,
            ),
        );
        let effective_height = i16::max(
            0,
            i16::min(i16::min(src_h - clip_y, dst_h - clip_y), self.rows as i16 - effective_dst_y),
        );

        // If there's nothing to copy (zero width or height), return early
        if effective_width <= 0 || effective_height <= 0 {
            return;
        }

        // Calculate effective src positions (accounting for clipping)
        let effective_src_x = src_x + clip_x;
        let effective_src_y = src_y + clip_y;

        // Copy the tiles row by row
        for y in 0..effective_height {
            for x in 0..effective_width {
                let src_index = (effective_src_y + y) as usize * columns as usize
                    + (effective_src_x + x) as usize;
                let dst_index = (effective_dst_y + y) as usize * self.columns as usize
                    + (effective_dst_x + x) as usize;
                let mut cell = cells[src_index];
                cell.id = TileID(cell.id.0 + tile_offset);
                self.cells[dst_index] = cell;
            }
        }
    }

    /// Copies a rectangular region from a source tilemap to this tilemap.
    ///
    /// If `src_rect` is None, copies the entire source. If `dst_rect` is None, pastes at (0,0).
    /// Negative destination coordinates are clipped automatically.
    pub fn copy_from(
        &mut self,
        src: &dyn DynTilemap,
        src_rect: Option<Rect<i16>>,
        dst_rect: Option<Rect<i16>>,
        tile_offset: u8,
    ) {
        self.copy_from_cells(
            src.cells(),
            src.columns() as i16,
            src.rows() as i16,
            src_rect,
            dst_rect,
            tile_offset,
        );
    }
}

impl<const CELL_COUNT: usize> DynTilemap for Tilemap<CELL_COUNT> {
    fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[inline(always)]
    fn get_index(&self, col: i16, row: i16) -> Option<usize> {
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
        CELL_COUNT
    }
}
