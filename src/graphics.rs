mod anim;
pub use anim::*;

mod text;
pub use text::*;

use crate::prelude::*;

pub fn world_to_view(
    pos: Vec2<i16>,
    scroll: Vec2<i16>,
    canvas_rect: Rect<i16>,
    canvas_scale: f32,
) -> Vec2<i16> {
    let x = ((pos.x as f32 - scroll.x as f32) * canvas_scale) as i16 + canvas_rect.x + 1;
    let y = ((pos.y as f32 - scroll.y as f32) * canvas_scale) as i16 + canvas_rect.y + 1;
    Vec2 { x, y }
}

pub fn view_to_world(
    pos: Vec2<i16>,
    scroll: Vec2<i16>,
    canvas_rect: Rect<i16>,
    canvas_scale: f32,
) -> Vec2<i16> {
    let x = ((pos.x - canvas_rect.x - 1) as f32 / canvas_scale) as i16 + scroll.x;
    let y = ((pos.y - canvas_rect.y - 1) as f32 / canvas_scale) as i16 + scroll.y;
    Vec2 { x, y }
}

/// Obtains the frame index on a given Animation based on the video chip's
/// internal frame counter.
pub fn anim_get_frame(current_video_frame: usize, anim: &Anim) -> usize {
    if anim.frames.is_empty() {
        return 0;
    }

    let len = anim.frames.len();
    let fps = anim.fps.max(1) as usize;
    let frame_duration = 60 / fps; // Assuming 60fps base
    let total_duration = frame_duration * len;

    if anim.repeat {
        let cycle = current_video_frame % total_duration;
        let frame_idx = cycle / frame_duration;
        frame_idx.min(len - 1)
    } else {
        let frame_idx = current_video_frame / frame_duration;
        frame_idx.min(len - 1)
    }
}

/// Clears a rectangular area in a tilemap with a specific cell.
pub fn tilemap_clear_rect<const LEN: usize>(bg: &mut Tilemap<LEN>, rect: Rect<i16>, cell: Cell) {
    for row in rect.y..rect.y + rect.h {
        for col in rect.x..rect.x + rect.w {
            bg.set_op(BgOp { col, row, cell });
        }
    }
}

/// Fills the entire tilemap with a specific tile.
pub fn tilemap_fill<const LEN: usize>(bg: &mut Tilemap<LEN>, cell: Cell) {
    let rect = Rect { x: 0, y: 0, w: bg.columns as i16, h: bg.rows as i16 };
    tilemap_clear_rect(bg, rect, cell);
}

/// Draws a sprite as a tilemap to the foreground layer.
/// The tilemap can be from a const strip or any other source.
pub fn draw_sprite_to_fg(video: &mut VideoChip, anim: &Anim, bundle: SpriteBundle) {
    let frame_number = anim_get_frame(video.frame_number, &anim);
    let strip_frame = anim.frames[frame_number] as usize;
    let tilemap = anim.strip[strip_frame];
    video.draw_sprite(bundle, &tilemap);
}

// TODO: Draw from Anim, just like draw_sprite_to_fg, but to the BG
/// Draws a tilemap to a background tilemap.
/// Positions the sprite at tile coordinates (not pixel coordinates).
pub fn draw_sprite_to_tilemap<const LEN: usize>(
    dest: &mut Tilemap<LEN>,
    src: &dyn DynTilemap,
    bundle: SpriteBundle,
) {
    let col = bundle.x / TILE_SIZE as i16;
    let row = bundle.y / TILE_SIZE as i16;
    let dst_rect = Rect {
        x: col,
        y: row,
        w: src.columns() as i16,
        h: src.rows() as i16,
    };
    dest.copy_from(src, None, Some(dst_rect), bundle.tile_offset);
}

/// Copies a rectangular area from a tilemap into another.
/// If any rect is "None" the entire map is used.
pub fn draw_tilemap_to_tilemap<const LEN: usize>(
    dest: &mut Tilemap<LEN>,
    dest_rect: Option<Rect<i16>>,
    src: &dyn DynTilemap,
    src_rect: Option<Rect<i16>>,
    tile_offset: u8,
) {
    dest.copy_from(src, src_rect, dest_rect, tile_offset);
}

/// Draws a "3x3 patch" (sometimes called "9-Patch") into a tilemap.
/// Each cell represents a corner, an edge, or the center tile.
/// The pattern is:
/// ```text
/// top_left,    top,        top_right,
/// left,        center,     right,
/// bottom_left, bottom,     bottom_right
/// ```
pub fn draw_patch_to_tilemap<const LEN: usize>(
    bg: &mut Tilemap<LEN>,
    rect: Rect<i16>,
    patch: &dyn DynTilemap,
    tile_offset: u8,
) {
    debug_assert!(patch.columns() == 3 && patch.rows() == 3, "invalid patch dimensions");

    let Some(top_left) = patch.get_cell(0, 0) else { return };
    bg.set_op(BgOp {
        col: rect.x,
        row: rect.y,
        cell: Cell {
            id: TileID(top_left.id.0 + tile_offset),
            flags: top_left.flags,
            colors: top_left.colors,
        },
    });

    let high_x = (rect.x + rect.w - 1).min(i16::MAX);
    let Some(top) = patch.get_cell(1, 0) else { return };
    for col in rect.x + 1..high_x {
        bg.set_op(BgOp {
            col,
            row: rect.y,
            cell: Cell {
                id: TileID(top.id.0 + tile_offset),
                flags: top.flags,
                colors: top.colors,
            },
        });
    }

    let Some(top_right) = patch.get_cell(2, 0) else { return };
    bg.set_op(BgOp {
        col: high_x,
        row: rect.y,
        cell: Cell {
            id: TileID(top_right.id.0 + tile_offset),
            flags: top_right.flags,
            colors: top_right.colors,
        },
    });

    let high_y = (rect.y + rect.h - 1).min(i16::MAX);
    let Some(left) = patch.get_cell(0, 1) else { return };
    for row in rect.y + 1..high_y {
        bg.set_op(BgOp {
            col: rect.x,
            row,
            cell: Cell {
                id: TileID(left.id.0 + tile_offset),
                flags: left.flags,
                colors: left.colors,
            },
        });
    }

    let Some(center) = patch.get_cell(1, 1) else { return };
    for row in rect.y + 1..high_y {
        for col in rect.x + 1..high_x {
            bg.set_op(BgOp {
                col,
                row,
                cell: Cell {
                    id: TileID(center.id.0 + tile_offset),
                    flags: center.flags,
                    colors: center.colors,
                },
            });
        }
    }

    let Some(right) = patch.get_cell(2, 1) else { return };
    for row in rect.y + 1..high_y {
        bg.set_op(BgOp {
            col: high_x,
            row,
            cell: Cell {
                id: TileID(right.id.0 + tile_offset),
                flags: right.flags,
                colors: right.colors,
            },
        });
    }

    let Some(bottom_left) = patch.get_cell(0, 2) else { return };
    bg.set_op(BgOp {
        col: rect.x,
        row: high_y,
        cell: Cell {
            id: TileID(bottom_left.id.0 + tile_offset),
            flags: bottom_left.flags,
            colors: bottom_left.colors,
        },
    });

    let Some(bottom) = patch.get_cell(1, 2) else { return };
    for col in rect.x + 1..high_x {
        bg.set_op(BgOp {
            col,
            row: high_y,
            cell: Cell {
                id: TileID(bottom.id.0 + tile_offset),
                flags: bottom.flags,
                colors: bottom.colors,
            },
        });
    }

    let Some(bottom_right) = patch.get_cell(2, 2) else { return };
    bg.set_op(BgOp {
        col: high_x,
        row: high_y,
        cell: Cell {
            id: TileID(bottom_right.id.0 + tile_offset),
            flags: bottom_right.flags,
            colors: bottom_right.colors,
        },
    });
}

/// Draws a text string to a target Tilemap, using a tilemap as a character font.
/// Returns the resulting height (in rows), if any.
pub fn draw_text<const LEN: usize>(
    target: &mut Tilemap<LEN>,
    target_col: i16,
    target_row: i16,
    op: &TextOp,
    text: &str,
) -> Option<i16> {
    debug_assert!(text.is_ascii());
    let mut cursor_x = 0;
    let mut cursor_y = 0;

    // Helper to draw a single character
    let mut draw_char = |ch: char, cursor_x: i16, cursor_y: i16| {
        let char_index = match op.character_set {
            CharacterSet::Long => char_set_long(ch) as usize,
            CharacterSet::Short => char_set_short(ch) as usize,
            CharacterSet::Arcade => char_set_arcade(ch) as usize,
        };
        let font_cols = op.font.columns() as usize;
        let col = char_index % font_cols;
        let row = char_index / font_cols;
        if let Some(cell) = op.font.get_cell(col as i16, row as i16) {
            target.set_op(BgOp {
                col: target_col + cursor_x,
                row: target_row + cursor_y,
                cell: Cell {
                    id: TileID(cell.id.0 + op.tile_offset), // TODO: This may overflow...
                    flags: cell.flags,
                    colors: op.colors,
                },
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
