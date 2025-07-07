use tato::prelude::*;
use tato_raylib::*;

mod patch;
use patch::*;

mod smileys;
use smileys::*;

const MAP_LEN: usize = 1024;

// Rects use "number of tiles" as the dimensions
fn main() {
    let mut bg_map = BGMap::<MAP_LEN>::new(32, 32);

    let mut tato = Tato::new(240, 180);
    tato.video.bg_color = RGBA12::new(1, 2, 3, 7);

    // Populate tilesets
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let _transparent = tato.banks[0].push_color(RGBA12::new(0, 0, 0, 0));
    let _empty_palette = tato.new_subpalette(0, [BG_COLOR, BLACK, GRAY, WHITE]);

    let tileset_smileys = tato.new_tileset(0, SMILEYS_TILESET).unwrap();
    let map_smileys = tato.new_tilemap(tileset_smileys, &SMILEYS_MAP);
    bg_copy(
        &tato.tilemap::<160>(map_smileys),
        None,
        &mut bg_map,
        Some(Rect { x: 3, y: 5, w: 16, h: 10 }),
    );

    let tileset_patch = tato.new_tileset(0, PATCH_TILESET).unwrap();
    let map_patch = tato.new_tilemap(tileset_patch, &PATCH_MAP);
    tato.draw_patch(&mut bg_map, map_patch, Rect { x: 1, y: 1, w: 20, h: 4 });

    // Backend
    let mut backend = RaylibBackend::new(&tato, 60.0);
    backend.bg_color = raylib::prelude::Color::BLACK;
    while !backend.ray.window_should_close() {
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

        backend.render(&mut tato, &[&bg_map]);
    }
}
