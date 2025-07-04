use backend_raylib::*;
use tato::prelude::*;

// mod patch;
// use patch::*;

// mod smileys;
// use smileys::*;

mod testmap;
use testmap::*;

const MAP_LEN:usize = 1024;

// Rects use "number of tiles" as the dimensions
fn main() {
    let mut tato = Tato::new(288, 216);
    let mut bg_map = BGMap::<MAP_LEN>::new(32, 32);
    tato.bg = Some(&mut bg_map);
    tato.video.bg_color = RGBA12::new(0, 0, 0, 0);

    // Populate tilesets
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let _transparent = tato.banks[0].push_color(RGBA12::new(0, 0, 0, 0));
    let _empty_palette = tato.new_subpalette(0, [BG_COLOR, BLACK, GRAY, WHITE]);

    // let tileset_smileys = tato.new_tileset(0, SMILEYS_TILESET).unwrap();
    // let map_smileys = tato.new_tilemap(tileset_smileys, &SMILEYS_MAP);
    // tato.draw_map(map_smileys, None, Some(Rect { x: 3, y: 5, w: 16, h: 10 }));

    let tileset_test = tato.new_tileset(0, TESTMAP_TILESET).unwrap();
    let map_test = tato.new_tilemap::<MAP_LEN>(tileset_test, &TEST_MAP);
    tato.draw_map(map_test, None, None);

    // let tileset_patch = tato.new_tileset(0, PATCH_TILESET).unwrap();
    // let map_patch = tato.new_tilemap(tileset_patch, &PATCH_MAP);
    // tato.draw_patch(map_patch, Rect { x: 1, y: 1, w: 20, h: 4 });

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

        backend.render(&mut tato);
    }
}
