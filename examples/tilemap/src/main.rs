use backend_raylib::{
    raylib::{self, color::Color, ffi::TraceLogLevel, texture::Image},
    *,
};
use tato::prelude::*;

mod patch;
use patch::*;

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

fn main() {
    let mut tato = Tato::new(240, 180);
    tato.video.bg_color = ColorID(0);

    // WARNING: build script is also adding default tiles, just for debugging
    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let tileset = tato.new_tileset(0, PATCH_TILESET).unwrap();

    let map_a = tato.new_tilemap(tileset, 3, &PATCH_MAP);
    // let map_b = tato.new_tilemap(tileset, 3, &DEFAULT_TILES_MAP);

    let w = 7;
    let h = 5;
    for row in 0..4 {
        for col in 0..2 {
            tato.draw_patch(
                Rect { x: (col * w) + 1, y: (row * h) + 1, w: w - 1, h: h - 1 },
                map_a, //
            );
        }
        // for col in 2..4 {
        //     tato.draw_patch(
        //         Rect { x: (col * w) + 1, y: (row * h) + 1, w: w - 1, h: h - 1 },
        //         map_b, //
        //     );
        // }
    }

    // Raylib setup
    let target_fps = 60.0;
    let w = tato.video.width() as i32;
    let h = tato.video.height() as i32;
    let (mut ray, ray_thread) = raylib::init()
        .log_level(TraceLogLevel::LOG_WARNING)
        .size(w * 3, h * 3)
        .title("Tato Demo")
        .vsync()
        .resizable()
        .build();
    config_raylib();
    ray.set_target_fps(target_fps as u32);

    // Create texture for rendering
    let mut pixels: [u8; W * H * 4] = core::array::from_fn(|_| 0);
    let mut render_texture = {
        let render_image = Image::gen_image_color(w, h, Color::BLACK);
        ray.load_texture_from_image(&ray_thread, &render_image).unwrap()
    };

    // Main Loop
    while !ray.window_should_close() {
        // Update backends
        copy_pixels_to_texture(&mut tato, &ray_thread, &mut ray, &mut pixels, &mut render_texture);
    }
}
