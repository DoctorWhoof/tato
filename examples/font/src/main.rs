use tato::{Tato, prelude::*};
use tato_raylib::*;

fn main() -> TatoResult<()> {
    let mut bg_map = Tilemap::<896>::new(32, 28);
    let mut tato = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new();

    // Graphics setup
    let _empty = tato.push_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let ts_font = tato.push_tileset(0, FONT_TILESET)?;

    let plt_default = tato.new_subpalette(0, [0, 14, 2, 2]);
    let plt_light = tato.new_subpalette(0, [0, 3, 2, 2]);
    let plt_cycle = tato.new_subpalette(0, [0, 3, 2, 0]);

    tato.video.bg_color = RGBA12::new(1, 2, 3);

    // Pre-draw fixed text (writes to BG Map)
    let mut line = 1;
    let col = 1;
    let height = tato
        .draw_text(
            &mut bg_map,
            "\"draw_text\" simply sets BG Map tiles, so they will scroll with \
        the rest of the map! Use the arrow keys to try it out.",
            TextOp {
                font: &FONT_MAP,
                id: ts_font,
                col,
                row: line,
                width: 27,
                palette: plt_light,
            },
        )
        .unwrap();

    line += height + 1;
    tato.draw_text(
        &mut bg_map,
        "0123456789",
        TextOp {
            font: &FONT_MAP,
            id: ts_font,
            col,
            row: line,
            width: 26,
            palette: plt_light,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextOp {
            font: &FONT_MAP,
            id: ts_font,
            col,
            row: line,
            width: 26,
            palette: plt_default,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "abcdefghijklmnopqrstuvwxyz",
        TextOp {
            font: &FONT_MAP,
            id: ts_font,
            col,
            row: line,
            width: 26,
            palette: plt_light,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        ":;<=>? !\"#$%&\'()*+,-./",
        TextOp {
            font: &FONT_MAP,
            id: ts_font,
            col,
            row: line,
            width: 26,
            palette: plt_default,
        },
    );

    // Animated text
    line += 2;
    tato.draw_text(
        &mut bg_map,
        "Animated palette",
        TextOp {
            font: &FONT_MAP,
            id: ts_font,
            col,
            row: line,
            width: 26,
            palette: plt_cycle,
        },
    );

    // Main Loop
    let mut cycle = 1.0;
    tato.video.wrap_bg = true;
    let mut backend = RaylibBackend::new(&tato);
    while !backend.ray.window_should_close() {
        // Input
        tato.frame_start(backend.ray.get_frame_time());
        backend.update_input(&mut tato.pad);

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
        tato.frame_finish();
        backend.render_canvas(&tato, &[&bg_map]);
        backend.render_dashboard(&mut dash, &tato);
        backend.present();
    }
    Ok(())
}
