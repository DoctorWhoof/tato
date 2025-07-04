use tato_layout::Rect;

use crate::*;

impl<'a> Tato<'a> {
    pub fn draw_patch(&mut self, map_id: MapID, rect: Rect<u16>) {
        let map = &self.assets.maps[map_id.0 as usize];

        assert!(map.columns == 3, err!("Patch tilemaps must be 3 columns wide."));
        assert!(map.rows == 3, err!("Patch rows must be 3 rows tall."));

        let Some(bg) = &mut self.bg else { return };
        let top_left = self.assets.cells[map.data_start as usize];
        bg.set_cell(BgOp {
            col: rect.x,
            row: rect.y,
            tile_id: top_left.id,
            flags: top_left.flags,
        });

        let top = self.assets.cells[map.data_start as usize + 1];
        for col in rect.x + 1..rect.x + rect.w {
            bg.set_cell(BgOp { col, row: rect.y, tile_id: top.id, flags: top.flags });
        }

        let top_right = self.assets.cells[map.data_start as usize + 2];
        bg.set_cell(BgOp {
            col: rect.x + rect.w,
            row: rect.y,
            tile_id: top_right.id,
            flags: top_right.flags,
        });

        let left = self.assets.cells[map.data_start as usize + 3];
        for row in rect.y + 1..rect.y + rect.h {
            bg.set_cell(BgOp { col: rect.x, row, tile_id: left.id, flags: left.flags });
        }

        let center = self.assets.cells[map.data_start as usize + 4];
        for row in rect.y + 1..rect.y + rect.h {
            for col in rect.x + 1..rect.x + rect.w {
                bg.set_cell(BgOp { col, row, tile_id: center.id, flags: center.flags });
            }
        }

        let right = self.assets.cells[map.data_start as usize + 5];
        for row in rect.y + 1..rect.y + rect.h {
            bg.set_cell(BgOp {
                col: rect.x + rect.w,
                row,
                tile_id: right.id,
                flags: right.flags,
            });
        }

        let bottom_left = self.assets.cells[map.data_start as usize + 6];
        bg.set_cell(BgOp {
            col: rect.x,
            row: rect.y + rect.h,
            tile_id: bottom_left.id,
            flags: bottom_left.flags,
        });

        let bottom = self.assets.cells[map.data_start as usize + 7];
        for col in rect.x + 1..rect.x + rect.w {
            bg.set_cell(BgOp {
                col,
                row: rect.y + rect.h,
                tile_id: bottom.id,
                flags: bottom.flags,
            });
        }

        let bottom_right = self.assets.cells[map.data_start as usize + 8];
        bg.set_cell(BgOp {
            col: rect.x + rect.w,
            row: rect.y + rect.h,
            tile_id: bottom_right.id,
            flags: bottom_right.flags,
        });
    }

    // #[inline]
    // pub(crate) fn get_map(&mut self, bank: u8, map: MapID) -> (MapEntry, &mut [Cell]) {
    //     let map_entry = self.banks[bank as usize].maps[map.0 as usize];
    //     let start = map_entry.data_start as usize;
    //     let end = start + map_entry.data_len as usize;
    //     let map = &mut self.banks[bank as usize].mem.bg.data[start..end];
    //     (map_entry, map)
    // }

    // #[inline]
    // pub(crate) fn set_tile(
    //     map_entry: MapEntry,
    //     map: &mut [Cell],
    //     col: u16,
    //     row: u16,
    //     tile_id: TileID,
    //     flags: TileFlags,
    // ) {
    //     let index = (row as usize * map_entry.columns as usize) + col as usize;
    //     map[index].id = tile_id;
    //     map[index].flags = flags;
    // }

    // pub fn draw_tile(&mut self, op: MapOp) {
    //     let (map_entry, map) = self.get_map(op.bank, op.map);
    //     Self::set_tile(map_entry, map, op.col, op.row, op.tile_id, op.flags);
    //     // TODO: Extend Map and MapEntry with functions like this
    //     // let rows = map.len() / map_entry.columns as usize;
    //     // if op.col as usize >= map_entry.columns as usize || op.row as usize >= rows as usize {
    //     //     return;
    //     // }

    //     // let index = (op.row as usize * map_entry.columns as usize) + op.col as usize;
    //     // map[index].id = op.tile_id;
    //     // map[index].flags = op.flags;
    // }

    /// Copies a rectangular region from a source tilemap to this tilemap.
    /// - If `src_rect` is None, attempts to copy the entire source tilemap.
    /// - If `dst_rect` is None, pastes at (0,0) and fills as many tiles as possible.
    /// - Negative destination coordinates are handled by clipping the source region.
    pub fn draw_map(
        &mut self,
        id: MapID,
        src_rect: Option<Rect<u16>>,
        dst_rect: Option<Rect<u16>>,
    ) {
        if id.0 >= self.assets.map_head {
            return;
        }

        let source = &self.assets.maps[id.0 as usize];
        let start = source.data_start as usize;
        let end = start + source.data_len as usize;
        let cells = &mut self.assets.cells[start..end];

        // Determine source rectangle
        let src_x = src_rect.map_or(0, |r| r.x) as i16;
        let src_y = src_rect.map_or(0, |r| r.y) as i16;
        let src_w = src_rect.map_or(source.columns, |r| r.w) as i16;
        let src_h = src_rect.map_or(source.rows, |r| r.h) as i16;

        // Make sure source rectangle is within bounds
        let src_w = i16::min(src_w, source.columns as i16 - src_x);
        let src_h = i16::min(src_h, source.rows as i16 - src_y);

        // Determine destination rectangle
        let dst_x = dst_rect.map_or(0, |r| r.x) as i16;
        let dst_y = dst_rect.map_or(0, |r| r.y) as i16;

        // Calculate clipping for negative coordinates
        let clip_x = if dst_x < 0 { -dst_x } else { 0 };
        let clip_y = if dst_y < 0 { -dst_y } else { 0 };

        // Adjust source and destination starting points
        let effective_src_x = src_x + clip_x;
        let effective_src_y = src_y + clip_y;
        let effective_dst_x = i16::max(0, dst_x);
        let effective_dst_y = i16::max(0, dst_y);

        let Some(bg) = &mut self.bg else { return };

        // Calculate effective width and height after clipping
        let effective_width =
            i16::max(0, i16::min(src_w - clip_x, bg.columns() as i16 - effective_dst_x));
        let effective_height =
            i16::max(0, i16::min(src_h - clip_y, bg.rows() as i16 - effective_dst_y));

        // If there's nothing to copy (zero width or height), return early
        if effective_width <= 0 || effective_height <= 0 {
            return;
        }

        // Copy the tiles row by row
        for y in 0..effective_height {
            for x in 0..effective_width {
                let src_index = (effective_src_y + y) as usize * source.columns as usize
                    + (effective_src_x + x) as usize;
                let dst_index = (effective_dst_y + y) as usize * bg.columns() as usize
                    + (effective_dst_x + x) as usize;

                // Ensure we're within bounds (additional safety check)
                // if src_index < S_LEN && dst_index < LEN {
                // cells[dst_index] = cells[src_index];
                bg.cells_mut()[dst_index] = cells[src_index];
                // }
            }
        }
    }
}
