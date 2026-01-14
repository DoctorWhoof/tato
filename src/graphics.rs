mod text;
pub use text::*;

use crate::prelude::*;

#[inline]
/// Obtains the frame index on a given Animation based on the video chip's
/// internal frame counter.
pub fn anim_get_frame<const LEN: usize>(
    current_video_frame: usize,
    frames: &[u8; LEN],
    fps: u8,
    repeat: bool,
) -> usize {
    if frames.is_empty() {
        return 0;
    }

    let fps = fps.max(1) as usize;
    let frame_duration = 60 / fps; // Assuming 60fps base
    let total_duration = frame_duration * LEN;

    if repeat {
        let cycle = current_video_frame % total_duration;
        let frame_idx = cycle / frame_duration;
        frame_idx.min(LEN - 1)
    } else {
        let frame_idx = current_video_frame / frame_duration;
        frame_idx.min(LEN - 1)
    }
}

/// Clears a rectangular area in a tilemap with a specific tile.
pub fn tilemap_clear_rect<const LEN: usize>(
    bg: &mut Tilemap<LEN>,
    rect: Rect<i16>,
    tile_id: TileID,
) {
    for row in rect.y..rect.y + rect.h {
        for col in rect.x..rect.x + rect.w {
            bg.set_op(BgOp {
                col,
                row,
                tile_id,
                flags: TileFlags::default(),
                color_mapping: 0,
            });
        }
    }
}

/// Fills the entire tilemap with a specific tile.
pub fn tilemap_fill<const LEN: usize>(bg: &mut Tilemap<LEN>, tile_id: TileID) {
    let rect = Rect { x: 0, y: 0, w: bg.columns as i16, h: bg.rows as i16 };
    tilemap_clear_rect(bg, rect, tile_id);
}

/// Drawing functions and graphics helpers.
impl Tato {
    /// Draws a tilemap as a sprite to the foreground layer.
    /// The tilemap can be from a const strip or any other source.
    pub fn draw_tilemap_to_fg(&mut self, tilemap: &dyn DynTilemap, bundle: SpriteBundle) {
        self.video.draw_sprite(bundle, tilemap);
    }

    /// Copies a rectangular area from a tilemap into another.
    /// If any rect is "None" the entire map is used.
    pub fn draw_tilemap_to_tilemap<const DEST_LEN: usize>(
        &self,
        dest: &mut Tilemap<DEST_LEN>,
        dest_rect: Option<Rect<u16>>,
        src: &dyn DynTilemap,
        src_rect: Option<Rect<u16>>,
    ) {
        dest.copy_from(src, src_rect, dest_rect);
    }

    /// Draws a tilemap to a background tilemap.
    /// Positions the sprite at tile coordinates (not pixel coordinates).
    pub fn draw_tilemap_to_bg<const LEN: usize>(
        &self,
        dest: &mut Tilemap<LEN>,
        src: &dyn DynTilemap,
        bundle: SpriteBundle,
    ) {
        let col = (bundle.x / TILE_SIZE as i16) as u16;
        let row = (bundle.y / TILE_SIZE as i16) as u16;
        let dst_rect = Rect { x: col, y: row, w: src.columns(), h: src.rows() };
        dest.copy_from(src, None, Some(dst_rect));
    }

    /// Draws a "3x3 patch" (sometimes called "9-Patch") into a tilemap.
    /// Each cell represents a corner, an edge, or the center tile.
    /// The pattern is:
    /// ```text
    /// top_left,    top,        top_right,
    /// left,        center,     right,
    /// bottom_left, bottom,     bottom_right
    /// ```
    pub fn draw_patch_3x3<const LEN: usize>(
        &mut self,
        bg: &mut Tilemap<LEN>,
        rect: Rect<i16>,
        patch: &dyn DynTilemap,
    ) {
        debug_assert!(patch.columns() == 3 && patch.rows() == 3, "invalid patch dimensions");

        let Some(top_left) = patch.get_cell(0, 0) else { return };
        bg.set_op(BgOp {
            col: rect.x,
            row: rect.y,
            tile_id: top_left.id,
            flags: top_left.flags,
            color_mapping: top_left.color_mapping,
        });

        let high_x = (rect.x + rect.w).min(i16::MAX);
        let Some(top) = patch.get_cell(1, 0) else { return };
        for col in rect.x + 1..high_x {
            bg.set_op(BgOp {
                col,
                row: rect.y,
                tile_id: top.id,
                flags: top.flags,
                color_mapping: top.color_mapping,
            });
        }

        let Some(top_right) = patch.get_cell(2, 0) else { return };
        bg.set_op(BgOp {
            col: high_x,
            row: rect.y,
            tile_id: top_right.id,
            flags: top_right.flags,
            color_mapping: top_right.color_mapping,
        });

        let high_y = (rect.y + rect.h).min(i16::MAX);
        let Some(left) = patch.get_cell(0, 1) else { return };
        for row in rect.y + 1..high_y {
            bg.set_op(BgOp {
                col: rect.x,
                row,
                tile_id: left.id,
                flags: left.flags,
                color_mapping: left.color_mapping,
            });
        }

        let Some(center) = patch.get_cell(1, 1) else { return };
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

        let Some(right) = patch.get_cell(2, 1) else { return };
        for row in rect.y + 1..high_y {
            bg.set_op(BgOp {
                col: high_x,
                row,
                tile_id: right.id,
                flags: right.flags,
                color_mapping: right.color_mapping,
            });
        }

        let Some(bottom_left) = patch.get_cell(0, 2) else { return };
        bg.set_op(BgOp {
            col: rect.x,
            row: high_y,
            tile_id: bottom_left.id,
            flags: bottom_left.flags,
            color_mapping: bottom_left.color_mapping,
        });

        let Some(bottom) = patch.get_cell(1, 2) else { return };
        for col in rect.x + 1..high_x {
            bg.set_op(BgOp {
                col,
                row: high_y,
                tile_id: bottom.id,
                flags: bottom.flags,
                color_mapping: bottom.color_mapping,
            });
        }

        let Some(bottom_right) = patch.get_cell(2, 2) else { return };
        bg.set_op(BgOp {
            col: high_x,
            row: high_y,
            tile_id: bottom_right.id,
            flags: bottom_right.flags,
            color_mapping: bottom_right.color_mapping,
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
                    // tile_id: TileID(cell.id.0 + tile_start),
                    tile_id: TileID(cell.id.0),
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
}
