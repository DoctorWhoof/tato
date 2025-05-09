use raylib::{color::Color, texture::Image, ffi::TraceLogLevel};
use tato::{Tato, prelude::*};

mod backend_raylib;
use backend_raylib::*;

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

fn main() {
    let mut tato = Tato::new(W as u16, H as u16);

    // Graphics setup
    let plt_default = tato
        .video
        .push_subpalette([BG_COLOR, LIGHT_BLUE, GRAY, GRAY]);
    let plt_light = tato.video.push_subpalette([BG_COLOR, WHITE, GRAY, GRAY]);
    let plt_cycle = tato.video.push_subpalette([BG_COLOR, WHITE, GRAY, BLACK]);
    tato.tiles.new_tile(&TILE_EMPTY);
    tato.video.bg_color = DARK_BLUE;
    tato.video.bg.columns = 32;
    tato.video.bg.rows = 24;

    // Load font. TODO: Streamline this.
    for tile in tato::fonts::TILESET_FONT.chunks(64) {
        tato.tiles.new_tile(tile);
    }

    // Pre-draw fixed text (writes to BG Map)
    let mut line = 1;
    let col = 1;
    let height = tato.draw_text(
        "\"draw_text\" simply sets BG Map tiles, so they will scroll with \
        the rest of the map! Use the arrow keys to try it out.",
        TextBundle {
            initial_font_tile: 1,
            col,
            row: line,
            width: 27,
            palette: plt_light,
        },
    );

    line += height + 1;
    tato.draw_text(
        "0123456789",
        TextBundle {
            initial_font_tile: 1,
            col,
            row: line,
            width: 26,
            palette: plt_light,
        },
    );

    line += 2;
    tato.draw_text(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        TextBundle {
            initial_font_tile: 1,
            col,
            row: line,
            width: 26,
            palette: plt_default,
        },
    );

    line += 2;
    tato.draw_text(
        "abcdefghijklmnopqrstuvwxyz",
        TextBundle {
            initial_font_tile: 1,
            col,
            row: line,
            width: 26,
            palette: plt_light,
        },
    );

    line += 2;
    tato.draw_text(
        ":;<=>? !\"#$%&\'()*+,-./",
        TextBundle {
            initial_font_tile: 1,
            col,
            row: line,
            width: 26,
            palette: plt_default,
        },
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
        ray.load_texture_from_image(&ray_thread, &render_image)
            .unwrap()
    };

    // Main Loop
    line += 2;
    let mut cycle = 1.0;
    tato.video.wrap_bg = true;

    while !ray.window_should_close() {
        // Input
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
        let color = &mut tato.video.local_palettes[plt_cycle.0 as usize][1];
        color.0 = cycle as u8;
        cycle += ray.get_frame_time() * 2.0;
        if cycle >= 16.0 {
            cycle = 1.0
        }

        tato.draw_text(
            "Animated palette",
            TextBundle {
                initial_font_tile: 1,
                col,
                row: line,
                width: 26,
                palette: plt_cycle,
            },
        );

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
