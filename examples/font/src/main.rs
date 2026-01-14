use tato::default_assets::*;
use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};
use tato_raylib::*;

fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<65_536, u32>::new();
    let mut bg_map = Tilemap::<896>::new(32, 28);
    let mut tato = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new().unwrap();
    let mut banks = [Bank::new()];

    // Graphics setup
    tato.video.bg_color = RGBA12::new(1, 2, 3);
    tato.video.bg_tile_bank = 0;
    banks[0].load_default_colors();
    banks[0].add_tile(&Tile::default());
    banks[0].append(&BANK_FONT_LONG).unwrap();

    banks[0].color_mapping[1][1] = 0;
    banks[0].color_mapping[1][3] = 14;

    banks[0].color_mapping[2][1] = 0;
    banks[0].color_mapping[2][3] = 3;

    banks[0].color_mapping[3][1] = 0;

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
                col,
                row: line,
                width,
                color_mapping: 2,
            },
        )
        .unwrap();

    line += height + 1;
    tato.draw_text(
        &mut bg_map,
        "0123456789",
        TextOp {
            font: &FONT_LONG_MAP,
            col,
            row: line,
            width,
            color_mapping: 1,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextOp {
            font: &FONT_LONG_MAP,
            col,
            row: line,
            width,
            color_mapping: 1,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        "abcdefghijklmnopqrstuvwxyz",
        TextOp {
            font: &FONT_LONG_MAP,
            col,
            row: line,
            width,
            color_mapping: 1,
        },
    );

    line += 2;
    tato.draw_text(
        &mut bg_map,
        ":;<=>? !\"#$%&\'()*+,-./",
        TextOp {
            font: &FONT_LONG_MAP,
            col,
            row: line,
            width,
            color_mapping: 1,
        },
    );

    // Animated text
    line += 2;
    tato.draw_text(
        &mut bg_map,
        "Animated palette",
        TextOp {
            font: &FONT_LONG_MAP,
            col,
            row: line,
            width,
            color_mapping: 3,
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
        let color = &mut banks[0].color_mapping[3][3];
        *color = cycle as u8;
        cycle += backend.ray.get_frame_time() * 4.0;
        if cycle >= 16.0 {
            cycle = 1.0
        }

        // Update backends
        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &banks, &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, &banks, &[&bg_map]);
    }
    Ok(())
}
