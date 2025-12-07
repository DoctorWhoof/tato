use tato::{arena::Arena, prelude::*};
use tato_raylib::*;
use tato::default_assets::*;

fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<32_768, u32>::new();
    let mut bg_map = Tilemap::<896>::new(32, 28);
    let mut tato = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new().unwrap();

    // tato.character_set = CharacterSet::Short;

    // Graphics setup
    let _empty = tato.push_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let ts_font = tato.push_tileset(0, FONT_LONG_TILESET)?;

    tato.video.bg_color = RGBA12::new(1, 2, 3);
    tato.banks[0].load_default_colors();

    // Pre-draw fixed text (writes to BG Map)
    let mut line = 1;
    let col = 1;
    let width = Some(26);
    let height = tato
        .draw_text(
            &mut bg_map,
            "\"draw_text\" simply sets BG Map tiles, so they will scroll with \
        the rest of the map! Use the arrow keys to try it out.",
            TextOp {
                font: &FONT_LONG_MAP,
                tileset: ts_font,
                col,
                row: line,
                width,
                // palette_override: Some(plt_light),
                color_mapping: 0
            },
        )
        .unwrap();

    line += height + 1;
    tato.draw_text(
        &mut bg_map,
        "0123456789",
        TextOp {
            font: &FONT_LONG_MAP,
            tileset: ts_font,
            col,
            row: line,
            width,
            // palette_override: Some(plt_light),
            color_mapping: 0
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextOp {
            font: &FONT_LONG_MAP,
            tileset: ts_font,
            col,
            row: line,
            width,
            // palette_override: Some(plt_default),
            color_mapping: 0
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "abcdefghijklmnopqrstuvwxyz",
        TextOp {
            font: &FONT_LONG_MAP,
            tileset: ts_font,
            col,
            row: line,
            width,
            // palette_override: Some(plt_light),
            color_mapping: 0
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        ":;<=>? !\"#$%&\'()*+,-./",
        TextOp {
            font: &FONT_LONG_MAP,
            tileset: ts_font,
            col,
            row: line,
            width,
            // palette_override: Some(plt_default),
            color_mapping: 0,
        },
    );

    // Animated text
    line += 2;
    tato.draw_text(
        &mut bg_map,
        "Animated palette",
        TextOp {
            font: &FONT_LONG_MAP,
            tileset: ts_font,
            col,
            row: line,
            width,
            // palette_override: Some(plt_cycle),
            color_mapping: 0
        },
    );

    // Main Loop
    let mut cycle = 1.0;
    tato.video.wrap_bg = true;
    let mut backend = RayBackend::new(&tato);
    while !backend.ray.window_should_close() {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);

        dash.frame_start(&mut frame_arena, &mut backend);
        tato.frame_start(backend.ray.get_frame_time());

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
        // let color = &mut tato.banks[0].sub_palettes[plt_cycle.0 as usize][1];
        // color.0 = cycle as u8;
        // cycle += backend.ray.get_frame_time() * 2.0;
        // if cycle >= 16.0 {
        //     cycle = 1.0
        // }

        // Update backends
        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &mut backend, &tato);
        backend.frame_present(&mut frame_arena, &tato, &[&bg_map]);
    }
    Ok(())
}
