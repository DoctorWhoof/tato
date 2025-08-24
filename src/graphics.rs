mod text;
pub use text::*;

use crate::prelude::*;

/// Drawing functions and graphics helpers.
impl Tato {
    /// Obtains the frame index on a given Animation based on the video chip's
    /// internal frame counter.
    #[inline(always)]
    pub fn get_anim_frame(&self, anim: AnimID) -> usize {
        // AnimID(0) means no animation - return 0 as default frame
        if anim.0 == 0 {
            return 0;
        }
        let Some(anim_entry) = self.assets.anim_entries.get(anim.0 as usize) else {
            return 0;
        };
        self.get_frame_from_anim_entry(anim_entry)
    }

    /// Copies a rectangular area from a tilemap into another.
    /// If any rect is "None" the entire map is used.
    pub fn draw_tilemap_to<const LEN: usize>(
        &self,
        dest: &mut Tilemap<LEN>,
        dest_rect: Option<Rect<u16>>,
        src: MapID,
        src_rect: Option<Rect<u16>>,
    ) {
        let Ok(src) = self.get_tilemap(src) else { return };
        dest.copy_from(&src, src_rect, dest_rect);
    }

    /// Draws a sprite's current frame (calculated using the video chip's
    /// internal time counter).
    pub fn draw_anim(&mut self, anim: AnimID, bundle: SpriteBundle) {
        // AnimID(0) means no animation - don't draw anything
        if anim.0 == 0 {
            return;
        }
        let Some(anim_entry) = self.assets.anim_entries.get(anim.0 as usize) else {
            return;
        };
        let Some(strip_entry) = self.assets.strip_entries.get(anim_entry.strip.0 as usize) else {
            return;
        };

        let base_index = self.get_frame_from_anim_entry(anim_entry);
        let Ok(frames) = self.assets.arena.get_slice(&anim_entry.frames) else { return };
        let Some(anim_index) = frames.get(base_index) else { return };
        let start_index = strip_entry.start_index;
        let index = start_index + anim_index;
        let Some(map_entry) = self.assets.map_entries.get(index as usize) else { return };

        debug_assert!(
            (index as usize) < strip_entry.start_index as usize + strip_entry.frame_count as usize,
            err!("Animation frame exceeds strip length")
        );

        let Ok(cells) = self.assets.arena.get_slice(&map_entry.cells) else { return };
        self.video.draw_sprite(
            bundle,
            &TilemapRef { cells, columns: map_entry.columns, rows: map_entry.rows },
        );
    }

    /// Draws a "patch" (sometimes called "9-Patch") into a tilemap.
    /// Patches are always 3x3 tilemaps, where each cell
    /// can represent a corner, and edge or the center tile.
    pub fn draw_patch<const LEN: usize>(
        &mut self,
        bg: &mut Tilemap<LEN>,
        bg_rect: Rect<i16>,
        patch_id: MapID,
    ) {
        // let map = &self.assets.map_entries[map_id.0 as usize];
        let Ok(map) = self.get_tilemap(patch_id) else { return };

        if map.columns != 3 || map.rows != 3 {
            return; // Silently return for invalid patch dimensions
        }

        let top_left = map.cells[0];
        bg.set_cell(BgOp {
            col: bg_rect.x,
            row: bg_rect.y,
            tile_id: top_left.id,
            flags: top_left.flags,
            sub_palette: top_left.sub_palette,
        });

        if (bg_rect.x as usize + bg_rect.w as usize) >= u16::MAX as usize {
            return; // Prevent overflow
        }
        let top = map.cells[1];
        for col in bg_rect.x + 1..bg_rect.x + bg_rect.w {
            bg.set_cell(BgOp {
                col,
                row: bg_rect.y,
                tile_id: top.id,
                flags: top.flags,
                sub_palette: top.sub_palette,
            });
        }

        let top_right = map.cells[2];
        bg.set_cell(BgOp {
            col: bg_rect.x + bg_rect.w,
            row: bg_rect.y,
            tile_id: top_right.id,
            flags: top_right.flags,
            sub_palette: top_right.sub_palette,
        });

        let left = map.cells[3];

        for row in bg_rect.y + 1..bg_rect.y + bg_rect.h {
            bg.set_cell(BgOp {
                col: bg_rect.x,
                row,
                tile_id: left.id,
                flags: left.flags,
                sub_palette: left.sub_palette,
            });
        }

        if (bg_rect.y as usize + bg_rect.h as usize) >= u16::MAX as usize {
            return; // Prevent overflow
        }
        let center = map.cells[4];
        for row in bg_rect.y + 1..bg_rect.y + bg_rect.h {
            for col in bg_rect.x + 1..bg_rect.x + bg_rect.w {
                bg.set_cell(BgOp {
                    col,
                    row,
                    tile_id: center.id,
                    flags: center.flags,
                    sub_palette: center.sub_palette,
                });
            }
        }

        let right = map.cells[5];
        for row in bg_rect.y + 1..bg_rect.y + bg_rect.h {
            bg.set_cell(BgOp {
                col: bg_rect.x + bg_rect.w,
                row,
                tile_id: right.id,
                flags: right.flags,
                sub_palette: right.sub_palette,
            });
        }

        let bottom_left = map.cells[6];
        bg.set_cell(BgOp {
            col: bg_rect.x,
            row: bg_rect.y + bg_rect.h,
            tile_id: bottom_left.id,
            flags: bottom_left.flags,
            sub_palette: bottom_left.sub_palette,
        });

        let bottom = map.cells[7];
        for col in bg_rect.x + 1..bg_rect.x + bg_rect.w {
            bg.set_cell(BgOp {
                col,
                row: bg_rect.y + bg_rect.h,
                tile_id: bottom.id,
                flags: bottom.flags,
                sub_palette: bottom.sub_palette,
            });
        }

        let bottom_right = map.cells[8];
        bg.set_cell(BgOp {
            col: bg_rect.x + bg_rect.w,
            row: bg_rect.y + bg_rect.h,
            tile_id: bottom_right.id,
            flags: bottom_right.flags,
            sub_palette: bottom.sub_palette,
        });
    }

    /// Draws a text string to a target Tilemap, using a tilemap as a character font.
    /// Returns the resulting height (in rows), if any.
    pub fn draw_text<const LEN: usize>(
        &mut self,
        target: &mut Tilemap<LEN>,
        text: &str,
        op: TextOp,
    ) -> Option<i16> {
        debug_assert!(text.is_ascii());
        let tileset = self.assets.tilesets.get(op.id.0 as usize)?;
        let tile_start = tileset.tile_start;
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        // Helper to draw a single character
        let mut draw_char = |ch: char, cursor_x: i16, cursor_y: i16| {
            let char_index = char_to_id_ex(ch) as usize;
            let font_cols = op.font.columns() as usize;
            let col = char_index % font_cols;
            let row = char_index / font_cols;
            if let Some(cell) = op.font.get_cell(col as i16, row as i16) {
                target.set_cell(BgOp {
                    col: op.col + cursor_x,
                    row: op.row + cursor_y,
                    tile_id: TileID(cell.id.0 + tile_start),
                    flags: cell.flags,
                    sub_palette: op.palette,
                });
            }
        };

        for word in text.split(' ') {
            if cursor_x + (word.len() as i16) > op.width {
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

    // Internal way to get the current frame from an already obtained AnimEntry
    #[inline(always)]
    fn get_frame_from_anim_entry(&self, anim: &AnimEntry) -> usize {
        assert!(anim.fps > 0, "Animation FPS must be higher than zero");
        let frame_duration = 1.0 / anim.fps as f32;
        ((self.time() as f32 / frame_duration) % anim.frames.len() as f32) as usize
    }
}
