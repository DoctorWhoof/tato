use tato_raylib::*;
use tato::{Tato, prelude::*};

fn main() {
    let mut bg_map = BGMap::<896>::new(32, 28);
    let mut tato = Tato::new(240, 180);
    tato.bg[0] = Some(&mut bg_map);

    // Graphics setup
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let font = tato.new_tileset(0, FONT_TILESET).unwrap();

    let plt_default = tato.new_subpalette(0, [BG_COLOR, LIGHT_BLUE, GRAY, GRAY]);
    let plt_light = tato.new_subpalette(0, [BG_COLOR, WHITE, GRAY, GRAY]);
    let plt_cycle = tato.new_subpalette(0, [BG_COLOR, WHITE, GRAY, BLACK]);

    tato.video.bg_color = RGBA12::new(1, 2, 3, 7);

    // Pre-draw fixed text (writes to BG Map)
    let mut line = 1;
    let col = 1;
    let height = tato
        .draw_text(
            0,
            "\"draw_text\" simply sets BG Map tiles, so they will scroll with \
        the rest of the map! Use the arrow keys to try it out.",
            TextOp { id: font, col, row: line, width: 27, palette: plt_light },
        )
        .unwrap();

    line += height + 1;
    tato.draw_text(
        0,
        "0123456789",
        TextOp { id: font, col, row: line, width: 26, palette: plt_light },
    );

    line += 2;
    tato.draw_text(
        0,
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextOp { id: font, col, row: line, width: 26, palette: plt_default },
    );

    line += 2;
    tato.draw_text(
        0,
        "abcdefghijklmnopqrstuvwxyz",
        TextOp { id: font, col, row: line, width: 26, palette: plt_light },
    );

    line += 2;
    tato.draw_text(
        0,
        ":;<=>? !\"#$%&\'()*+,-./",
        TextOp { id: font, col, row: line, width: 26, palette: plt_default },
    );

    // Animated text
    line += 2;
    tato.draw_text(
        0,
        "Animated palette",
        TextOp { id: font, col, row: line, width: 26, palette: plt_cycle },
    );

    // Main Loop
    let mut cycle = 1.0;
    tato.video.wrap_bg = true;
    let mut backend = RaylibBackend::new(&tato, 60.0);
    while !backend.ray.window_should_close() {
        // Input
        tato.video.start_frame();
        backend.update_gamepad(&mut tato.pad);

        if tato.pad.is_down(Button::Right) {
            tato.video.scroll_x += 1;
        } else if tato.pad.is_down(Button::Left) {
            tato.video.scroll_x -= 1;
        }

        if tato.pad.is_down(Button::Down) {
            tato.video.scroll_y += 1;
        } else if tato.pad.is_down(Button::Up) {
            tato.video.scroll_y -= 1;
        }

        // Draw
        let color = &mut tato.banks[0].sub_palettes[plt_cycle.0 as usize][1];
        color.0 = cycle as u8;
        cycle += backend.ray.get_frame_time() * 2.0;
        if cycle >= 16.0 {
            cycle = 1.0
        }

        // Update backends
        backend.render(&mut tato);
    }
}
