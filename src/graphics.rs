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
    pub fn draw_tilemap<const LEN: usize>(
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
    pub fn draw_anim_to_fg(&mut self, anim: AnimID, bundle: SpriteBundle) {
        if let Some(map_entry) = self.get_sprite_tilemap_entry(anim) {
            if let Ok(cells) = self.assets.arena.get_slice(&map_entry.cells) {
                self.video.draw_sprite(
                    bundle,
                    &TilemapRef { cells, columns: map_entry.columns, rows: map_entry.rows },
                );
            };
        };
    }

    // TODO: Implement flipping
    pub fn draw_anim_to_bg<const LEN: usize>(
        &self,
        dest: &mut Tilemap<LEN>,
        anim: AnimID,
        bundle: SpriteBundle,
    ) {
        if let Some(map_entry) = self.get_sprite_tilemap_entry(anim) {
            let Some(src) = map_entry.to_ref(&self.assets.arena) else {
                println!("Invalid map entry from {:?}", anim);
                return;
            };
            let col = (bundle.x / TILE_SIZE as i16) as u16;
            let row = (bundle.y / TILE_SIZE as i16) as u16;
            let dst_rect = Rect { x: col, y: row, w: src.columns, h: src.rows };
            dest.copy_from(&src, None, Some(dst_rect));
        };
    }

    /// Draws a "3x3 patch" (sometimes called "9-Patch") into a tilemap.
    /// Each cell can represent a corner, an edge or the center tile.
    /// The pattern is:
    /// top_left,    top,        top_right,
    /// left,        center,     right,
    /// bottom_left, bottom,     bottom_right
    pub fn draw_patch_3x3<const LEN: usize>(
        &mut self,
        bg: &mut Tilemap<LEN>,
        rect: Rect<i16>,
        patch_id: MapID,
    ) {
        let Ok(map) = self.get_tilemap(patch_id) else { return };

        debug_assert!(map.columns == 3 && map.rows == 3, "invalid patch dimensions");

        let top_left = map.cells[0];
        bg.set_op(BgOp {
            col: rect.x,
            row: rect.y,
            tile_id: top_left.id,
            flags: top_left.flags,
            color_mapping: top_left.color_mapping,
        });

        let high_x = (rect.x + rect.w).min(i16::MAX); // Prevent overflow
        let top = map.cells[1];
        for col in rect.x + 1..high_x {
            bg.set_op(BgOp {
                col,
                row: rect.y,
                tile_id: top.id,
                flags: top.flags,
                color_mapping: top.color_mapping,
            });
        }

        let top_right = map.cells[2];
        bg.set_op(BgOp {
            col: rect.x + rect.w,
            row: rect.y,
            tile_id: top_right.id,
            flags: top_right.flags,
            color_mapping: top_right.color_mapping,
        });

        let left = map.cells[3];

        for row in rect.y + 1..rect.y + rect.h {
            bg.set_op(BgOp {
                col: rect.x,
                row,
                tile_id: left.id,
                flags: left.flags,
                color_mapping: left.color_mapping,
            });
        }

        let high_y = (rect.y + rect.h).min(i16::MAX); // Prevent overflow
        let center = map.cells[4];
        for row in rect.y + 1..high_y {
            for col in rect.x + 1..high_x {
                bg.set_op(BgOp {
                    col,
                    row,
                    tile_id: center.id,
                    flags: center.flags,
                    color_mapping: center.color_mapping,
                });
            }
        }

        let right = map.cells[5];
        for row in rect.y + 1..high_y {
            bg.set_op(BgOp {
                col: high_x,
                row,
                tile_id: right.id,
                flags: right.flags,
                color_mapping: right.color_mapping,
            });
        }

        let bottom_left = map.cells[6];
        bg.set_op(BgOp {
            col: rect.x,
            row: high_y,
            tile_id: bottom_left.id,
            flags: bottom_left.flags,
            color_mapping: bottom_left.color_mapping,
        });

        let bottom = map.cells[7];
        for col in rect.x + 1..high_x {
            bg.set_op(BgOp {
                col,
                row: high_y,
                tile_id: bottom.id,
                flags: bottom.flags,
                color_mapping: bottom.color_mapping,
            });
        }

        let bottom_right = map.cells[8];
        bg.set_op(BgOp {
            col: high_x,
            row: high_y,
            tile_id: bottom_right.id,
            flags: bottom_right.flags,
            color_mapping: bottom.color_mapping,
        });
    }

    // // UNTESTED
    // /// Draws a "patch" with a single row of tiles into a tilemap.
    // /// The pattern is:
    // /// [top, bottom],
    // /// where middle is optional
    // pub fn draw_patch_1x2<const LEN: usize>(
    //     &mut self,
    //     bg: &mut Tilemap<LEN>,
    //     rect: Rect<i16>,
    //     patch_id: MapID,
    // ) {
    //     let Ok(map) = self.get_tilemap(patch_id) else { return };

    //     debug_assert!(map.columns == 1 && map.rows == 2, "invalid patch dimensions");

    //     let high_x = (rect.x + rect.w).min(i16::MAX); // Prevent overflow
    //     let high_y = (rect.y + rect.h).min(i16::MAX); // Prevent overflow

    //     let top = map.cells[0];
    //     for col in rect.x + 1..high_x {
    //         bg.set_op(BgOp {
    //             col,
    //             row: rect.y,
    //             tile_id: top.id,
    //             flags: top.flags,
    //             color_mapping: top.color_mapping,
    //         });
    //     }

    //     let bottom = map.cells[1];
    //     for col in rect.x + 1..high_x {
    //         bg.set_op(BgOp {
    //             col,
    //             row: high_y,
    //             tile_id: bottom.id,
    //             flags: bottom.flags,
    //             color_mapping: bottom.color_mapping,
    //         });
    //     }
    // }

    /// Draws a text string to a target Tilemap, using a tilemap as a character font.
    /// Returns the resulting height (in rows), if any.
    pub fn draw_text<const LEN: usize>(
        &mut self,
        target: &mut Tilemap<LEN>,
        text: &str,
        op: TextOp,
    ) -> Option<i16> {
        debug_assert!(text.is_ascii());
        let tileset = self.assets.tilesets.get(op.tileset.0 as usize)?;
        let tile_start = tileset.tile_start;
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        // Helper to draw a single character
        let mut draw_char = |ch: char, cursor_x: i16, cursor_y: i16| {
            let char_index = match self.character_set {
                CharacterSet::Long => char_set_long(ch) as usize,
                CharacterSet::Short => char_set_short(ch) as usize,
                CharacterSet::Arcade => char_set_arcade(ch) as usize,
            };
            let font_cols = op.font.columns() as usize;
            let col = char_index % font_cols;
            let row = char_index / font_cols;
            if let Some(cell) = op.font.get_cell(col as i16, row as i16) {
                target.set_op(BgOp {
                    col: op.col + cursor_x,
                    row: op.row + cursor_y,
                    tile_id: TileID(cell.id.0 + tile_start),
                    flags: cell.flags,
                    color_mapping: op.color_mapping,
                });
            }
        };

        let width = op.width.unwrap_or(255);
        for word in text.split(' ') {
            if cursor_x + (word.len() as i16) > width {
                cursor_x = 0;
                cursor_y += 1;
            }
            for ch in word.chars() {
                draw_char(ch, cursor_x, cursor_y);
                cursor_x += 1;
            }
            if cursor_x >= width {
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

    #[inline(always)]
    fn get_sprite_tilemap_entry(&self, anim: AnimID) -> Option<TilemapEntry> {
        if anim.0 == 0 {
            // AnimID(0) means no animation
            return None;
        }
        let anim_entry = self.assets.anim_entries.get(anim.0 as usize)?;
        let strip_entry = self.assets.strip_entries.get(anim_entry.strip.0 as usize)?;

        let base_index = self.get_frame_from_anim_entry(anim_entry);
        let frames = self.assets.arena.get_slice(&anim_entry.frames).ok()?;
        let anim_index = frames.get(base_index)?;
        let start_index = strip_entry.start_index;
        let index = start_index + anim_index;
        let map_entry = self.assets.map_entries.get(index as usize)?;
        Some(*map_entry)
    }

    // Internal way to get the current frame from an already obtained AnimEntry
    #[inline(always)]
    fn get_frame_from_anim_entry(&self, anim: &AnimEntry) -> usize {
        assert!(anim.fps > 0, "Animation FPS must be higher than zero");
        let frame_duration = 1.0 / anim.fps as f32;
        ((self.time() as f32 / frame_duration) % anim.frames.len() as f32) as usize
    }
}
