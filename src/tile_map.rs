use crate::*;

#[derive(Debug, Clone)]
pub struct Tilemap<const LEN: usize> {
    pub data: [TileEntry; LEN],
    pub columns: u16,
    pub rows: u16,
}

/// A simple packet of required data to fully set the attributes on a BG tile.
#[derive(Debug, Clone, Copy)]
pub struct BgBundle {
    pub col: u16,
    pub row: u16,
    pub tile_id: TileID,
    pub flags: TileFlags,
}

impl<const LEN: usize> Tilemap<LEN> {
    pub fn new(columns: u16, rows: u16) -> Self {
        assert!(
            (columns as usize * rows as usize) <= LEN,
            err!("Invalid tilemap dimensions")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("Tilemap dimensions can't be zero")
        );
        Self {
            data: core::array::from_fn(|_| TileEntry::default()),
            columns,
            rows,
        }
    }

    pub fn width(&self) -> u16 {
        self.columns as u16 * TILE_SIZE as u16
    }

    pub fn height(&self) -> u16 {
        self.rows as u16 * TILE_SIZE as u16
    }

    pub fn set_size(&mut self, columns: u16, rows: u16) {
        assert!(
            columns as usize * rows as usize <= LEN,
            err!("Invalid column count")
        );
        assert!(
            columns > 0 && rows > 0,
            err!("Tilemap dimensions can't be zero")
        );
        self.columns = columns;
        self.rows = rows;
    }

    // Returns None if coords are out of map. not sure if useful yet.
    // Iterator uses its own, wrapping coordinates.
    fn get_index(&self, col: u16, row: u16) -> Option<usize> {
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
            self.data[index].id = data.tile_id;
            self.data[index].flags = data.flags;
        }
    }

    pub fn set_id(&mut self, col: u16, row: u16, tile_id: impl Into<TileID>) {
        if let Some(index) = self.get_index(col, row) {
            self.data[index].id = tile_id.into();
        }
    }

    pub fn set_flags(&mut self, col: u16, row: u16, flags: impl Into<TileFlags>) {
        if let Some(index) = self.get_index(col, row) {
            self.data[index].flags = flags.into();
        }
    }

    pub fn get_id(&self, col: u16, row: u16) -> Option<TileID> {
        let index = self.get_index(col, row)?;
        Some(self.data[index].id)
    }

    pub fn get_flags(&self, col: u16, row: u16) -> Option<TileFlags> {
        let index = self.get_index(col, row)?;
        Some(self.data[index].flags)
    }

    // /// Copies a rectangular region from a source tilemap to this tilemap.
    // /// - If `src_rect` is None, attempts to copy the entire source tilemap.
    // /// - If `dst_rect` is None, pastes at (0,0) and fills as many tiles as possible.
    // /// - Negative destination coordinates are handled by clipping the source region.
    // pub fn copy_rect<const S_LEN: usize>(
    //     &mut self,
    //     source: &Tilemap<S_LEN>,
    //     src_rect: Option<&Rect>,
    //     dst_rect: Option<&Rect>,
    // ) {
    //     // Determine source rectangle
    //     let src_x = src_rect.map_or(0, |r| r.x) as i16;
    //     let src_y = src_rect.map_or(0, |r| r.y) as i16;
    //     let src_w = src_rect.map_or(source.columns, |r| r.w()) as i16;
    //     let src_h = src_rect.map_or(source.rows, |r| r.h()) as i16;

    //     // Make sure source rectangle is within bounds
    //     let src_w = i16::min(src_w, source.columns as i16 - src_x);
    //     let src_h = i16::min(src_h, source.rows as i16 - src_y);

    //     // Determine destination rectangle
    //     let dst_x = dst_rect.map_or(0, |r| r.x) as i16;
    //     let dst_y = dst_rect.map_or(0, |r| r.y) as i16;

    //     // Calculate clipping for negative coordinates
    //     let clip_x = if dst_x < 0 { -dst_x } else { 0 };
    //     let clip_y = if dst_y < 0 { -dst_y } else { 0 };

    //     // Adjust source and destination starting points
    //     let effective_src_x = src_x + clip_x;
    //     let effective_src_y = src_y + clip_y;
    //     let effective_dst_x = i16::max(0, dst_x);
    //     let effective_dst_y = i16::max(0, dst_y);

    //     // Calculate effective width and height after clipping
    //     let effective_width = i16::max(
    //         0,
    //         i16::min(src_w - clip_x, self.columns as i16 - effective_dst_x),
    //     );
    //     let effective_height = i16::max(
    //         0,
    //         i16::min(src_h - clip_y, self.rows as i16 - effective_dst_y),
    //     );

    //     // If there's nothing to copy (zero width or height), return early
    //     if effective_width <= 0 || effective_height <= 0 {
    //         return;
    //     }

    //     // Copy the tiles row by row
    //     for y in 0..effective_height {
    //         for x in 0..effective_width {
    //             let src_index = (effective_src_y + y) as usize * source.columns as usize
    //                 + (effective_src_x + x) as usize;
    //             let dst_index = (effective_dst_y + y) as usize * self.columns as usize
    //                 + (effective_dst_x + x) as usize;

    //             // Ensure we're within bounds (additional safety check)
    //             // if src_index < S_LEN && dst_index < LEN {
    //                 self.data[dst_index] = source.data[src_index];
    //             // }
    //         }
    //     }
    // }
}
