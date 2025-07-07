use tato_video::math::Rect;

use crate::*;

impl Tato {
    pub fn draw_patch(&mut self, bg: &mut dyn DynamicBGMap, map_id: MapID, rect: Rect<u16>) {
        let map = &self.assets.map_entries[map_id.0 as usize];

        assert!(map.columns == 3, err!("Patch tilemaps must be 3 columns wide."));
        assert!(map.rows == 3, err!("Patch rows must be 3 rows tall."));

        let top_left = self.assets.cells[map.data_start as usize];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x,
                row: rect.y,
                tile_id: top_left.id,
                flags: top_left.flags,
            },
        );

        let top = self.assets.cells[map.data_start as usize + 1];
        for col in rect.x + 1..rect.x + rect.w {
            bg_set_cell(bg, BgOp { col, row: rect.y, tile_id: top.id, flags: top.flags });
        }

        let top_right = self.assets.cells[map.data_start as usize + 2];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x + rect.w,
                row: rect.y,
                tile_id: top_right.id,
                flags: top_right.flags,
            },
        );

        let left = self.assets.cells[map.data_start as usize + 3];
        for row in rect.y + 1..rect.y + rect.h {
            bg_set_cell(bg, BgOp { col: rect.x, row, tile_id: left.id, flags: left.flags });
        }

        let center = self.assets.cells[map.data_start as usize + 4];
        for row in rect.y + 1..rect.y + rect.h {
            for col in rect.x + 1..rect.x + rect.w {
                bg_set_cell(bg, BgOp { col, row, tile_id: center.id, flags: center.flags });
            }
        }

        let right = self.assets.cells[map.data_start as usize + 5];
        for row in rect.y + 1..rect.y + rect.h {
            bg_set_cell(
                bg,
                BgOp {
                    col: rect.x + rect.w,
                    row,
                    tile_id: right.id,
                    flags: right.flags,
                },
            );
        }

        let bottom_left = self.assets.cells[map.data_start as usize + 6];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x,
                row: rect.y + rect.h,
                tile_id: bottom_left.id,
                flags: bottom_left.flags,
            },
        );

        let bottom = self.assets.cells[map.data_start as usize + 7];
        for col in rect.x + 1..rect.x + rect.w {
            bg_set_cell(
                bg,
                BgOp {
                    col,
                    row: rect.y + rect.h,
                    tile_id: bottom.id,
                    flags: bottom.flags,
                },
            );
        }

        let bottom_right = self.assets.cells[map.data_start as usize + 8];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x + rect.w,
                row: rect.y + rect.h,
                tile_id: bottom_right.id,
                flags: bottom_right.flags,
            },
        );
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
}
