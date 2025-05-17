use backend_raylib::{
    raylib::{self, color::Color, texture::Image},
    *,
};
use tato::{
    Tato,
    video::*,
    {TILE_EMPTY, TILESET_DEFAULT},
};

mod patch;
use patch::*;

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

fn main() {
    let mut tato = Tato::new(240, 180);
    let _empty = tato.tiles.new_tile(0, &TILESET_DEFAULT[TILE_EMPTY]);
    let tileset = tato.tiles.new_tileset(0, &TILESET_PATCH);
    let colors = [BG_COLOR, BLACK, DARK_BLUE, BLUE];
    tato.draw_patch(0, 4, 4, 8, 5, tileset);

    // Raylib setup
    let target_fps = 60.0;
    let w = tato.video.width() as i32;
    let h = tato.video.height() as i32;
    let (mut ray, ray_thread) = raylib::init()
        // .log_level(TraceLogLevel::LOG_WARNING)
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
        ray.load_texture_from_image(&ray_thread, &render_image)
            .unwrap()
    };

    // Main Loop
    while !ray.window_should_close() {
        // Update backends
        copy_pixels_to_texture(
            &mut tato,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );
    }
}
