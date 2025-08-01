mod text;
pub use text::*;

use crate::prelude::*;

impl Tato {

    // Internal way to get the current frame from an already obtained AnimEntry
    #[inline(always)]
    fn get_frame_from_anim_entry(&self, anim: &AnimEntry) -> usize {
        debug_assert!(anim.fps > 0, "Animation FPS must be higher than zero");
        let current_frame = self.video.frame_count() as f32;
        let time = current_frame * (1.0 / self.target_fps as f32);
        let frame_duration = 1.0 / anim.fps as f32;
        ((time / frame_duration) % anim.frames.len() as f32) as usize
    }

    // Public way to obtain the animation frame
    /// Obtains the frame index on a given Animation based on the video chip's
    /// internal frame counter.
    #[inline(always)]
    pub fn get_anim_frame(&self, anim:AnimID) -> usize {
        let anim_entry = self.assets.anim_entries.get(anim.0 as usize).unwrap();
        self.get_frame_from_anim_entry(anim_entry)
    }

    /// Draws a sprite's frame, which is calculated using the video chip's
    /// internal frame counter.
    pub fn draw_anim(&mut self, anim: AnimID, bundle: SpriteBundle) {
        let Some(anim_entry) = self.assets.anim_entries.get(anim.0 as usize) else {
            return;
        };
        let Some(strip_entry) = self.assets.strip_entries.get(anim_entry.strip_id.0 as usize)
        else {
            return;
        };

        let base_index = self.get_frame_from_anim_entry(anim_entry);
        let frames = self.assets.arena.get_pool(&anim_entry.frames);
        let Some(anim_index) = frames.get(base_index) else { return };
        let start_index = strip_entry.start_index;
        let index = start_index + anim_index;
        let Some(map_entry) = self.assets.map_entries.get(index as usize) else { return };

        debug_assert!(
            (index as usize) < strip_entry.start_index as usize + strip_entry.frame_count as usize,
            err!("Animation frame exceeds strip length")
        );

        self.video.draw_sprite(
            bundle,
            &TilemapRef {
                cells: self.assets.arena.get_pool(&map_entry.cells),
                columns: map_entry.columns,
                rows: map_entry.rows,
            },
        );
    }

    pub fn draw_patch<const LEN: usize>(
        &mut self,
        bg: &mut Tilemap<LEN>,
        map_id: MapID,
        rect: Rect<u16>,
    ) {
        // let map = &self.assets.map_entries[map_id.0 as usize];
        let map = self.get_tilemap(map_id);

        assert!(map.columns == 3, err!("Patch tilemaps must be 3 columns wide."));
        assert!(map.rows == 3, err!("Patch rows must be 3 rows tall."));

        let top_left = map.cells[0];
        bg.set_cell(BgOp {
            col: rect.x,
            row: rect.y,
            tile_id: top_left.id,
            flags: top_left.flags,
        });

        debug_assert!((rect.x as usize + rect.w as usize) < u16::MAX as usize);
        let top = map.cells[1];
        for col in rect.x + 1..rect.x + rect.w {
            bg.set_cell(BgOp { col, row: rect.y, tile_id: top.id, flags: top.flags });
        }

        let top_right = map.cells[2];
        bg.set_cell(BgOp {
            col: rect.x + rect.w,
            row: rect.y,
            tile_id: top_right.id,
            flags: top_right.flags,
        });

        let left = map.cells[3];

        for row in rect.y + 1..rect.y + rect.h {
            bg.set_cell(BgOp { col: rect.x, row, tile_id: left.id, flags: left.flags });
        }

        debug_assert!((rect.y as usize + rect.h as usize) < u16::MAX as usize);
        let center = map.cells[4];
        for row in rect.y + 1..rect.y + rect.h {
            for col in rect.x + 1..rect.x + rect.w {
                bg.set_cell(BgOp { col, row, tile_id: center.id, flags: center.flags });
            }
        }

        let right = map.cells[5];
        for row in rect.y + 1..rect.y + rect.h {
            bg.set_cell(BgOp {
                col: rect.x + rect.w,
                row,
                tile_id: right.id,
                flags: right.flags,
            });
        }

        let bottom_left = map.cells[6];
        bg.set_cell(BgOp {
            col: rect.x,
            row: rect.y + rect.h,
            tile_id: bottom_left.id,
            flags: bottom_left.flags,
        });

        let bottom = map.cells[7];
        for col in rect.x + 1..rect.x + rect.w {
            bg.set_cell(BgOp {
                col,
                row: rect.y + rect.h,
                tile_id: bottom.id,
                flags: bottom.flags,
            });
        }

        let bottom_right = map.cells[8];
        bg.set_cell(BgOp {
            col: rect.x + rect.w,
            row: rect.y + rect.h,
            tile_id: bottom_right.id,
            flags: bottom_right.flags,
        });
    }

    /// "Draws" a text string to the BG Map, returns the resulting height (in rows), if any.
    pub fn draw_text<const LEN: usize>(
        &mut self,
        bg: &mut Tilemap<LEN>,
        text: &str,
        op: TextOp,
    ) -> Option<u16> {
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
            if let Some(cell) = op.font.get_cell(col as u16, row as u16) {
                bg.set_cell(BgOp {
                    col: op.col + cursor_x,
                    row: op.row + cursor_y,
                    tile_id: TileID(cell.id.0 + tile_start),
                    flags: cell.flags.with_palette(op.palette),
                });
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
