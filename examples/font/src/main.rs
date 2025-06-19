use backend_raylib::{
    raylib::{color::Color, ffi::TraceLogLevel, texture::Image},
    *,
};
use tato::{Tato, prelude::*};

mod font;

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

fn main() {
    let mut tato = Tato::new(W as u16, H as u16);
    // tato.banks[0].bg.set_size(32, 24);

    // Graphics setup

    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]);
    let font = tato.new_tileset(0, FONT_TILESET).unwrap();
    let plt_default = tato.new_subpalette(0, [BG_COLOR, LIGHT_BLUE, GRAY, GRAY]);
    let plt_light = tato.new_subpalette(0, [BG_COLOR, WHITE, GRAY, GRAY]);
    let plt_cycle = tato.new_subpalette(0, [BG_COLOR, WHITE, GRAY, BLACK]);

    tato.video.bg_color = DARK_BLUE;

    // Pre-draw fixed text (writes to BG Map)
    let mut line = 1;
    let col = 1;
    let height = tato
        .draw_text(
            "\"draw_text\" simply sets BG Map tiles, so they will scroll with \
        the rest of the map! Use the arrow keys to try it out.",
            TextOp { id: font, col, row: line, width: 27, palette: plt_light },
        )
        .unwrap();

    line += height + 1;
    tato.draw_text(
        "0123456789",
        TextOp { id: font, col, row: line, width: 26, palette: plt_light },
    );

    line += 2;
    tato.draw_text(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextOp { id: font, col, row: line, width: 26, palette: plt_default },
    );

    line += 2;
    tato.draw_text(
        "abcdefghijklmnopqrstuvwxyz",
        TextOp { id: font, col, row: line, width: 26, palette: plt_light },
    );

    line += 2;
    tato.draw_text(
        ":;<=>? !\"#$%&\'()*+,-./",
        TextOp { id: font, col, row: line, width: 26, palette: plt_default },
    );

    // Animated text
    line += 2;
    tato.draw_text(
        "Animated palette",
        TextOp { id: font, col, row: line, width: 26, palette: plt_cycle },
    );

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
    let mut cycle = 1.0;
    tato.video.wrap_bg = true;

    while !ray.window_should_close() {
        // Input
        tato.video.start_frame();
        update_gamepad(&ray, &mut tato.pad);

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
        let color = &mut tato.banks[0].sub_palettes[plt_cycle.0 as usize][1];
        color.0 = cycle as u8;
        cycle += ray.get_frame_time() * 2.0;
        if cycle >= 16.0 {
            cycle = 1.0
        }

        // Update backends
        copy_pixels_to_texture(&mut tato, &ray_thread, &mut ray, &mut pixels, &mut render_texture);
    }
}
