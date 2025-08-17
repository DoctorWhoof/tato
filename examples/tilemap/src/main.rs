use tato::{arena::Arena, prelude::*};

use tato_raylib::*;

mod patch;
use patch::*;

mod smileys;
use smileys::*;

const MAP_LEN: usize = 1024;

// Rects use "number of tiles" as the dimensions
fn main() -> TatoResult<()> {
    let mut bg_map = Tilemap::<MAP_LEN>::new(32, 32);
    let mut arena = Arena::new();
    let mut dash = Dashboard::new(&mut arena);
    let mut tato = Tato::new(240, 180, 60);

    tato.video.bg_color = RGBA12::new(1, 2, 3);

    // Populate tilesets
    let _empty = tato.push_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let _transparent = tato.banks[0].push_color(RGBA12::TRANSPARENT);
    let _empty_palette = tato.new_subpalette(0, [0, 1, 2, 3]);

    let tileset_smileys = tato.push_tileset(0, SMILEYS_TILESET)?;
    let map_smileys = tato.load_tilemap(tileset_smileys, &SMILEYS_MAP)?;
    tato.draw_tilemap_to(&mut bg_map, Some(Rect { x: 3, y: 5, w: 16, h: 10 }), map_smileys, None);

    let tileset_patch = tato.push_tileset(0, PATCH_TILESET)?;
    let map_patch = tato.load_tilemap(tileset_patch, &PATCH_MAP)?;
    tato.draw_patch(&mut bg_map, Rect { x: 1, y: 1, w: 20, h: 4 }, map_patch);

    println!("Asset arena: {} Bytes", tato.assets.used_memory());
    // Backend
    let mut backend = RaylibBackend::new(&tato);
    backend.set_bg_color(RGBA32::BLACK);

    while !backend.ray.window_should_close() {
        arena.clear();
        tato.frame_start(backend.ray.get_frame_time());
        dash.start_frame(&mut arena);
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

        tato.frame_finish();
        backend.render_canvas(&tato, &[&bg_map]);
        backend.render_dashboard(&tato, &mut dash, &mut arena);
        backend.present(&tato, Some(&dash), &arena);
    }
    Ok(())
}
