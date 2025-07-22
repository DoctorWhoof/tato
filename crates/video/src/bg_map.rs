use crate::*;
use tato_math::rect::Rect;

/// This trait allows references to BGMaps of different sizes.
pub trait DynamicBGMap: core::fmt::Debug {
    fn cells(&self) -> &[Cell];
    fn cells_mut(&mut self) -> &mut [Cell];
    fn columns(&self) -> u16;
    fn rows(&self) -> u16;
    fn width(&self) -> u16;
    fn height(&self) -> u16;
    fn len(&self) -> usize;
    fn set_size(&mut self, columns: u16, rows: u16);
}

/// A simple collection of Cells, split into a number of columns and rows that can never exceed the
/// constant CELL_COUNT capacity.
#[derive(Debug, Clone)]
pub struct Tilemap<const CELL_COUNT: usize> {
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

impl<const CELL_COUNT: usize> Tilemap<CELL_COUNT> {
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
        assert!(
            columns > 0 && rows > 0,
            err!("Tilemap dimensions can't be zero")
        );
        Self {
            cells: core::array::from_fn(|_| Cell::default()),
            columns,
            rows,
        }
    }
}

impl<const CELL_COUNT: usize> DynamicBGMap for Tilemap<CELL_COUNT> {
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

    fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= self.cells().len(),
            err!("Invalid column count")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("Tilemap dimensions can't be zero")
        );
        self.columns = columns;
        self.rows = rows;
    }
}

// Standalone functions. Unfortunately I ran into borrow checker issues when
// using trait object methods!

#[inline(always)]
pub fn bg_get_index(map: &dyn DynamicBGMap, col: u16, row: u16) -> Option<usize> {
    if col as usize >= map.columns() as usize || row as usize >= map.rows() as usize {
        return None;
    }
    Some((row as usize * map.columns() as usize) + col as usize)
}

pub fn bg_set_cell(map: &mut dyn DynamicBGMap, op: BgOp) {
    if let Some(index) = bg_get_index(map, op.col, op.row) {
        map.cells_mut()[index].id = op.tile_id;
        map.cells_mut()[index].flags = op.flags;
    }
}

pub fn bg_set_id(map: &mut dyn DynamicBGMap, col: u16, row: u16, tile_id: TileID) {
    if let Some(index) = bg_get_index(map, col, row) {
        map.cells_mut()[index].id = tile_id;
    }
}

pub fn bg_set_flags(map: &mut dyn DynamicBGMap, col: u16, row: u16, flags: TileFlags) {
    if let Some(index) = bg_get_index(map, col, row) {
        map.cells_mut()[index].flags = flags;
    }
}

pub fn bg_get_cell(map: &dyn DynamicBGMap, col: u16, row: u16) -> Option<Cell> {
    let index = bg_get_index(map, col, row)?;
    Some(map.cells()[index])
}

pub fn bg_get_id(map: &dyn DynamicBGMap, col: u16, row: u16) -> Option<TileID> {
    let index = bg_get_index(map, col, row)?;
    Some(map.cells()[index].id)
}

pub fn bg_get_flags(map: &dyn DynamicBGMap, col: u16, row: u16) -> Option<TileFlags> {
    let index = bg_get_index(map, col, row)?;
    Some(map.cells()[index].flags)
}

/// Copies a rectangular region from a src tilemap to this tilemap.
/// - If `src_rect` is None, attempts to copy the entire source tilemap.
/// - If `dst_rect` is None, pastes at (0,0) and fills as many tiles as possible.
/// - Negative destination coordinates are handled by clipping the source region.
pub fn bg_copy(
    src: &dyn DynamicBGMap,
    src_rect: Option<Rect<u16>>,
    dst: &mut dyn DynamicBGMap,
    dst_rect: Option<Rect<u16>>,
) {
    // Determine source rectangle
    let src_x = src_rect.map_or(0, |r| r.x) as i16;
    let src_y = src_rect.map_or(0, |r| r.y) as i16;
    let src_w = src_rect.map_or(src.columns(), |r| r.w) as i16;
    let src_h = src_rect.map_or(src.rows(), |r| r.h) as i16;

    // Make sure source rectangle is within bounds
    let src_w = i16::min(src_w, src.columns() as i16 - src_x);
    let src_h = i16::min(src_h, src.rows() as i16 - src_y);

    // Determine destination rectangle
    let dst_x = dst_rect.map_or(0, |r| r.x) as i16;
    let dst_y = dst_rect.map_or(0, |r| r.y) as i16;

    // Calculate clipping for negative coordinates
    let clip_x = if dst_x < 0 { -dst_x } else { 0 };
    let clip_y = if dst_y < 0 { -dst_y } else { 0 };

    // Adjust source and destination starting points
    let effective_dst_x = i16::max(0, dst_x);
    let effective_dst_y = i16::max(0, dst_y);

    // Calculate effective width and height after clipping
    let effective_width = i16::max(
        0,
        i16::min(src_w - clip_x, dst.columns() as i16 - effective_dst_x),
    );
    let effective_height = i16::max(
        0,
        i16::min(src_h - clip_y, dst.rows() as i16 - effective_dst_y),
    );

    // If there's nothing to copy (zero width or height), return early
    if effective_width <= 0 || effective_height <= 0 {
        return;
    }

    // Calculate effective src positions (accounting for clipping)
    let effective_src_x = clip_x;
    let effective_src_y = clip_y;

    // Copy the tiles row by row
    for y in 0..effective_height {
        for x in 0..effective_width {
            let src_index = (effective_src_y + y) as usize * src.columns() as usize
                + (effective_src_x + x) as usize;
            let dst_index = (effective_dst_y + y) as usize * dst.columns() as usize
                + (effective_dst_x + x) as usize;
            dst.cells_mut()[dst_index] = src.cells()[src_index];
        }
    }
}
