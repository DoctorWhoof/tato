use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};
use tato_raylib::*;

mod assets;
use assets::*;

const MAP_LEN: usize = 1024;

// Rects use "number of tiles" as the dimensions
fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<65_536, u32>::new();
    let mut bg_map = Tilemap::<MAP_LEN>::new(32, 32);
    let mut dash = Dashboard::new().unwrap();
    let mut tato = Tato::new(240, 180, 60);
    let mut banks = [Bank::new()];

    tato.video.bg_color = RGBA12::with_transparency(7,5,0,7);
    tato.video.wrap_bg = true;

    // Combine multiple banks into bank 0
    banks[0].tiles.add(&Tile::default());
    // banks[0].colors.push_color(RGBA12::new(0, 0, 0));
    let patch_offset = banks[0].append(&BANK_PATCH).unwrap();
    // let patch_offset = banks[0].append_tiles(&TILES_PATCH).unwrap();
    // let smileys_offset = banks[0].append(&BANK_SMILEYS).unwrap();

    // Draw using the new direct tilemap API
    draw_patch_to_tilemap(&mut bg_map, Rect { x: 1, y: 1, w: 20, h: 4 }, &MAP_PATCH, patch_offset);
    // draw_tilemap_to_tilemap(
    //     &mut bg_map,
    //     Some(Rect { x: 3, y: 5, w: 16, h: 10 }),
    //     &MAP_SMILEYS,
    //     None,
    //     smileys_offset,
    // );

    // Backend
    let mut backend = RayBackend::new(&tato);
    // backend.set_bg_color(RGBA32::BLACK);

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

        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &banks, &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, &banks, &[&bg_map]);
    }
    Ok(())
}
