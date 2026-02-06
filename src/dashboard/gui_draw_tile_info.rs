use super::*;

impl Dashboard {
    pub(super) fn draw_tile_info<A>(
        &mut self,
        bg: &dyn DynTilemap,
        bg_bank: &Bank,
        arena: &mut A,
        tato: &Tato,
        backend: &impl Backend,
    ) where
        A: ArenaOps<u32, ()>,
    {
        let Some(mouse) = self.world_mouse(backend.get_mouse(), tato) else { return };

        let tile_size = TILE_SIZE as i16;
        let col = mouse.x / tile_size;
        let row = mouse.y / tile_size;
        if let Some(cell) = bg.get_cell(col, row) {
            let text = Text::from_str(arena, "------------- BG Tile Info -------------");
            self.push_text(arena, text.unwrap());

            let text = Text::format_display(arena, "TileID: {}", &[cell.id.0], "");
            self.push_text(arena, text.unwrap());

            self.push_display_txt(
                arena,
                "TileFlags:{},{},{},{},{}",
                &[
                    if cell.flags.is_flipped_x() { "X" } else { "  " },
                    if cell.flags.is_flipped_y() { "Y" } else { "  " },
                    if cell.flags.is_rotated() { "R" } else { "  " },
                    if cell.flags.is_collider() { "C" } else { "  " },
                    if cell.flags.is_fg() { "FG" } else { "  " },
                ],
                "",
            );

            self.push_display_txt(
                arena,
                "Colors:{},{},{},{}",
                &[cell.colors.get(0), cell.colors.get(1), cell.colors.get(2), cell.colors.get(3)],
                "",
            );

            // Highlight current BG tile with rect
            let rect = Rect {
                x: col * tile_size,
                y: row * tile_size,
                w: tile_size,
                h: tile_size,
            };
            self.draw_rect(arena, rect, RGBA12::WHITE, true, true);
        }
    }
}
