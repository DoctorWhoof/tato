mod text;
pub use text::*;

use crate::prelude::*;

impl Tato {
    #[inline(always)]
    pub fn get_anim_frame<const LEN: usize>(&mut self, anim: &Anim<LEN>) -> usize {
        let current_frame = self.video.frame_count() as f32;
        let time = current_frame * (1.0 / self.target_fps as f32);
        let frame_duration = 1.0 / anim.fps as f32;
        ((time / frame_duration) % anim.frames.len() as f32) as usize
    }

    pub fn draw_anim<const MAP_LEN: usize, const ANIM_LEN: usize>(
        &mut self,
        frames: &[Tilemap<MAP_LEN>],
        anim: &Anim<ANIM_LEN>,
        bundle: SpriteBundle,
    ) {
        let index = self.get_anim_frame(anim);
        let mapped_index = anim.frames[index] as usize;
        self.video.draw_sprite(bundle, &frames[mapped_index]);
    }

    pub fn draw_patch(&mut self, bg: &mut dyn DynTilemap, map_id: MapID, rect: Rect<u16>) {
        // let map = &self.assets.map_entries[map_id.0 as usize];
        let map = self.get_tilemap(map_id);

        assert!(map.columns == 3, err!("Patch tilemaps must be 3 columns wide."));
        assert!(map.rows == 3, err!("Patch rows must be 3 rows tall."));

        let top_left = map.cells[0];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x,
                row: rect.y,
                tile_id: top_left.id,
                flags: top_left.flags,
            },
        );

        let top = map.cells[1];
        for col in rect.x + 1..rect.x + rect.w {
            bg_set_cell(bg, BgOp { col, row: rect.y, tile_id: top.id, flags: top.flags });
        }

        let top_right = map.cells[2];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x + rect.w,
                row: rect.y,
                tile_id: top_right.id,
                flags: top_right.flags,
            },
        );

        let left = map.cells[3];
        for row in rect.y + 1..rect.y + rect.h {
            bg_set_cell(bg, BgOp { col: rect.x, row, tile_id: left.id, flags: left.flags });
        }

        let center = map.cells[4];
        for row in rect.y + 1..rect.y + rect.h {
            for col in rect.x + 1..rect.x + rect.w {
                bg_set_cell(bg, BgOp { col, row, tile_id: center.id, flags: center.flags });
            }
        }

        let right = map.cells[5];
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

        let bottom_left = map.cells[6];
        bg_set_cell(
            bg,
            BgOp {
                col: rect.x,
                row: rect.y + rect.h,
                tile_id: bottom_left.id,
                flags: bottom_left.flags,
            },
        );

        let bottom = map.cells[7];
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

        let bottom_right = map.cells[8];
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

    /// "Draws" a text string to the BG Map, returns the resulting height (in rows), if any.
    pub fn draw_text(&mut self, bg: &mut dyn DynTilemap, text: &str, op: TextOp) -> Option<u16> {
        debug_assert!(text.is_ascii());
        let tileset = self.assets.tilesets.get(op.id.0 as usize)?;
        let tile_start = tileset.tile_start;
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        // Helper to draw a single character
        let mut draw_char = |ch: char, cursor_x: u16, cursor_y: u16| {
            let char_index = char_to_id_ex(ch) as usize;
            let font_cols = op.font.columns() as usize;
            let col = char_index % font_cols;
            let row = char_index / font_cols;
            if let Some(cell) = bg_get_cell(op.font, col as u16, row as u16) {
                bg_set_cell(
                    bg,
                    BgOp {
                        col: op.col + cursor_x,
                        row: op.row + cursor_y,
                        tile_id: TileID(cell.id.0 + tile_start),
                        flags: cell.flags.with_palette(op.palette),
                    },
                );
            }
        };

        for word in text.split(' ') {
            if cursor_x + (word.len() as u16) > op.width {
                cursor_x = 0;
                cursor_y += 1;
            }
            for ch in word.chars() {
                draw_char(ch, cursor_x, cursor_y);
                cursor_x += 1;
            }
            if cursor_x >= op.width {
                cursor_x = 0;
                cursor_y += 1;
            } else {
                draw_char(' ', cursor_x, cursor_y);
                cursor_x += 1;
            }
        }

        // If successful, return number of lines written
        Some(cursor_y + 1)
    }
}
