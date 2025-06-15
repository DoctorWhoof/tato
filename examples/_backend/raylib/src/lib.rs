use tato::video::*;

pub use raylib;
use raylib::prelude::*;
use tato::{Tato, prelude::*};

pub fn config_raylib() {
    unsafe {
        raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
        // Disable ESC to close window. "Cmd + Q" still works!
        raylib::ffi::SetExitKey(raylib::ffi::KeyboardKey::KEY_NULL as i32);
    }
}

pub fn update_gamepad(ray: &RaylibHandle, pad: &mut AnaloguePad) {
    // Ensures gamepad can detect button changes
    pad.copy_current_to_previous_state();

    // Handle keyboard input
    if ray.is_key_down(KeyboardKey::KEY_LEFT) {
        pad.set_button(Button::Left, true);
    } else {
        pad.set_button(Button::Left, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_RIGHT) {
        pad.set_button(Button::Right, true);
    } else {
        pad.set_button(Button::Right, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_UP) {
        pad.set_button(Button::Up, true);
    } else {
        pad.set_button(Button::Up, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_DOWN) {
        pad.set_button(Button::Down, true);
    } else {
        pad.set_button(Button::Down, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_ESCAPE) {
        pad.set_button(Button::Menu, true);
    } else {
        pad.set_button(Button::Menu, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_ENTER) {
        pad.set_button(Button::Start, true);
    } else {
        pad.set_button(Button::Start, false);
    }

    if ray.is_key_down(KeyboardKey::KEY_Z) {
        pad.set_button(Button::A, true);
    } else {
        pad.set_button(Button::A, false);
    }
}

pub fn copy_pixels_to_texture(
    t: &mut Tato,
    thread: &RaylibThread,
    ray: &mut RaylibHandle,
    pixels: &mut [u8],
    texture: &mut Texture2D,
) {
    let time = std::time::Instant::now();

    // Copy from framebuffer to raylib texture
    for (color, coords) in t.video.iter_pixels(&t.get_video_banks(), &[&t.bg]) {
        let i = ((coords.y as usize * t.video.width() as usize) + coords.x as usize) * 4;
        pixels[i] = color.r;
        pixels[i + 1] = color.g;
        pixels[i + 2] = color.b;
        pixels[i + 3] = color.a;
    }
    texture.update_texture(pixels).unwrap();

    t.update_time_acc.push(time.elapsed().as_secs_f64() * 1000.0);
    println!("iter time: {:.2} ms", t.update_time_acc.average());

    let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
    let debug_w = (tiles_per_row * TILE_SIZE as usize) as i32;
    let debug_h = (((TILE_COUNT + tiles_per_row - 1) / tiles_per_row) * TILE_SIZE as usize) as i32;

    let mut tile_debug_texture = {
        let render_image = Image::gen_image_color(debug_w, debug_h, Color::BLACK);
        ray.load_texture_from_image(thread, &render_image).unwrap()
    };

    // Copy tile pixels to debug texture

    {
        let mut pixel_data = vec![0u8; (debug_w * debug_h * 4) as usize];

        for tile_index in 0..TILE_COUNT {
            let bank = &t.banks[0];
            let tile_x = tile_index % tiles_per_row;
            let tile_y = tile_index / tiles_per_row;

            for y in 0..TILE_SIZE as usize {
                for x in 0..TILE_SIZE as usize {
                    let color_index = bank.tiles[tile_index].get_pixel(x as u8, y as u8);
                    let gray_value = color_index * 63; // Map 0-4 to 0-252

                    let pixel_x = tile_x * TILE_SIZE as usize + x;
                    let pixel_y = tile_y * TILE_SIZE as usize + y;
                    let pixel_offset = (pixel_y * debug_w as usize + pixel_x) * 4;

                    // Set RGBA values
                    pixel_data[pixel_offset] = gray_value; // R
                    pixel_data[pixel_offset + 1] = gray_value; // G
                    pixel_data[pixel_offset + 2] = gray_value; // B
                    pixel_data[pixel_offset + 3] = 255; // A
                }
            }
        }

        tile_debug_texture.update_texture(&pixel_data).unwrap();
    }

    // Calculate rect with correct aspect ratio with integer scaling
    let screen_width = ray.get_screen_width();
    let screen_height = ray.get_screen_height();

    let scale = (screen_height as f32 / t.video.height() as f32).floor() as i32;
    let w = t.video.width() as i32 * scale;
    let h = t.video.height() as i32 * scale;
    let draw_rect_x = (screen_width - w) / 2;
    let draw_rect_y = (screen_height - h) / 2;

    // Present frame
    let mut canvas = ray.begin_drawing(thread);
    let bg_color = Color::new(64, 64, 64, 255);
    // let bg_color = Color::new(48, 32, 24, 255);
    canvas.clear_background(bg_color);
    canvas.draw_texture_ex(
        &texture,
        Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
        0.0,
        scale as f32,
        Color::WHITE,
    );
    let x = screen_width - debug_w as i32 - 8;
    let y = 8;
    canvas.draw_texture_ex(
        &tile_debug_texture,
        Vector2::new(x as f32, y as f32 + 16.0),
        0.0,
        1.0,
        Color::WHITE,
    );
    canvas.draw_text("bank 0", x, y, 16, Color::WHITE);
}
