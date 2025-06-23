use backend_raylib::*;
use tato::prelude::*;

mod assets;
use assets::*;

fn main() {
    let mut tato = Tato::new(240, 180);
    tato.video.bg_color = RGBA12::new(0, 0, 0, 0);

    // Populate tileset
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let _empty_palette = tato.new_subpalette(0, [BG_COLOR, BLACK, GRAY, WHITE]);
    let tileset = tato.new_tileset(0, PATCH_TILESET).unwrap();
    let map_patch = tato.new_tilemap(tileset, &PATCH_MAP);
    let map_smileys = tato.new_tilemap(tileset, &SMILEYS_MAP);

    // Rects use "number of tiles" as the dimensions
    tato.draw_patch(map_patch, Rect { x: 1, y: 1, w: 5, h: 4 });
    tato.draw_map(map_smileys, None, Some(Rect { x: 3, y: 5, w: 16, h: 10 }));

    // Backend
    let mut backend = RaylibBackend::new(&tato, 60.0);
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
