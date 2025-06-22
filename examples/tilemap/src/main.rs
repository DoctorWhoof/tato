use backend_raylib::*;
use tato::prelude::*;

mod assets;
use assets::*;

fn main() {
    let mut tato = Tato::new(240, 180);
    tato.video.bg_color = ColorID(0);

    // Populate tileset
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let tileset = tato.new_tileset(0, PATCH_TILESET).unwrap();
    let map_patch = tato.new_tilemap(tileset, &PATCH_MAP);
    let map_smileys = tato.new_tilemap(tileset, &SMILEYS_MAP);

    // Rects use "number of tiles" as the dimensions
    tato.draw_patch(map_patch, Rect { x: 1, y: 1, w: 5, h: 4 });
    tato.draw_map(map_smileys, None, Some(Rect { x: 3, y: 5, w: 16, h: 10 }));

    // Backend
    let mut backend = RaylibBackend::new(&tato, 60.0);
    while !backend.ray.window_should_close() {
        backend.render(&mut tato);
    }
}
