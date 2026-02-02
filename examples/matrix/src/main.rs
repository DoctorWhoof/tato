#![no_std]
// use tato::{arena::{Arena, ArenaOps}, default_assets::*, prelude::*};
// use tato_raylib::RayBackend;

// const COLUMNS: usize = 40;
// const ROWS: usize = 64;

fn main() {
// fn main() -> tato::TatoResult<()> {
    // // Init
    // let mut tato = Tato::new(320, 240, 60);
    // tato.video.bg_color = RGBA12::BLACK;
    // let color_bg = banks[0].push_color(RGBA12::with_transparency(0, 0, 0, 0));
    // let color_green_dark = banks[0].push_color(RGBA12::new(0, 3, 1));
    // let color_green_light = banks[0].push_color(RGBA12::new(1, 7, 3));

    // let mut bg = Tilemap::<2560>::new(COLUMNS as u16, ROWS as u16);
    // let tileset = tato.push_tileset(0, FONT_LONG_TILESET).unwrap();

    // let mut rng = tato::rng::Rng::new(32, 123);
    // let char_max = CHARACTER_SET_LONG.len() as u32;
    // let text: [Row; COLUMNS as usize] = core::array::from_fn(|_| {
    //     Row {
    //         head: 0,
    //         segment_len: (rng.next_u32() % 15) as u8,
    //         chars: core::array::from_fn(|_| 'x'),
    //     }
    //     // let len = ((rng.next_u32() % 12) + 3) as u8; //min 3, max 14
    //     // core::array::from_fn(|_| (rng.next_u32() % char_max) as u8)
    // });

    // // for col in 0..COLUMNS {
    // //     draw_chars(
    // //         &mut tato,
    // //         &mut bg,
    // //         TextOp {
    // //             font: &FONT_LONG_MAP,
    // //             tileset,
    // //             col: col as i16,
    // //             row: 0,
    // //             width: None,
    // //             palette_override: None,
    // //         },
    // //         &text[col],
    // //     );
    // // }

    // let mut dash = Dashboard::new().unwrap();
    // let mut temp = Arena::<32768>::new();
    // let mut backend = RayBackend::new(&tato);

    // while !backend.should_close() {
    //     // Frame start
    //     temp.clear();
    //     dash.frame_start(&mut temp, &mut backend);
    //     backend.frame_start(&mut temp, &mut tato.pad);
    //     tato.frame_start(backend.get_elapsed_time());

    //     // Update
    //     // tato.video.scroll_y = tato.video.scroll_y.wrapping_sub(1);

    //     // Finish
    //     tato.frame_finish();
    //     dash.frame_present(&mut temp, &mut backend, &tato);
    //     backend.frame_present(&mut temp, &tato, &[&bg]);
    // }

    // Ok(())
}

// -----------------------------------------------------------------------------

// struct Row {
//     head: u8,
//     segment_len: u8,
//     chars: [char; ROWS],
// }

// fn fill_chars(text: &mut [Row; COLUMNS], rng: &mut Rng) {
//     for row in text {
//         if row.segment_len == 0 {
//             //init row
//             row.segment_len = (rng.next_u32() % 12) as u8 + 3; // min=3, max=14
//         }
//     }
// }

// fn draw_chars<const LEN: usize>(tato: &mut Tato, bg: &mut Tilemap<LEN>, op: TextOp, row: Row) {
//     for i in 0..row.chars.len() as i16 {
//         let mut ch = [0; 4];
//         let ch_str = (row.chars[i as usize] as char).encode_utf8(&mut ch);
//         tato.draw_text(bg, ch_str, TextOp { row: op.row + i, ..op });
//     }
// }
