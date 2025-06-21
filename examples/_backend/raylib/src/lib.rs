use std::array::from_fn;

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

    if ray.is_key_down(KeyboardKey::KEY_ONE) {
        pad.set_button(Button::LeftShoulder, true);
    } else {
        pad.set_button(Button::LeftShoulder, false);
    }
}

pub fn tato_to_raylib(
    t: &mut Tato,
    thread: &RaylibThread,
    ray: &mut RaylibHandle,
    pixels: &mut [u8],
    texture: &mut Texture2D,
) {
    static mut DISPLAY_DEBUG: bool = true;

    let time = std::time::Instant::now();
    if ray.is_key_pressed(KeyboardKey::KEY_TAB) {
        unsafe { DISPLAY_DEBUG = !DISPLAY_DEBUG }
    }

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

    let debug_image = Image::gen_image_color(debug_w, debug_h, Color::BLACK);
    let mut tile_debug_texture: [Texture2D; BANK_COUNT] =
        from_fn(|_| ray.load_texture_from_image(thread, &debug_image).unwrap());

    // Calculate rect with correct aspect ratio with integer scaling
    let screen_width = ray.get_screen_width();
    let screen_height = ray.get_screen_height();

    let scale = (screen_height as f32 / t.video.height() as f32).floor() as i32;
    let w = t.video.width() as i32 * scale;
    let h = t.video.height() as i32 * scale;
    let draw_rect_x = (screen_width - w) / 2;
    let draw_rect_y = (screen_height - h) / 2;

    // Present pixels
    let mut canvas = ray.begin_drawing(thread);
    let bg_color = Color::new(64, 64, 64, 255);

    canvas.clear_background(bg_color);
    canvas.draw_texture_ex(
        &texture,
        Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
        0.0,
        scale as f32,
        Color::WHITE,
    );

    // Copy tile pixels to debug texture
    let mut y = 8;
    unsafe {
        if DISPLAY_DEBUG {
            for n in 0..BANK_COUNT {
                let mut pixel_data = vec![0u8; (debug_w * debug_h * 4) as usize];
                let x = screen_width - debug_w as i32 - 16;

                // acquire tile pixels.
                for tile_index in 0..TILE_COUNT {
                    let bank = &t.banks[n];
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

                // label
                let text = format!("bank {}", n);
                canvas.draw_text(&text, x, y, 8, Color::WHITE);

                // colors
                y += 10;
                let mut color_x = x;
                let gap = 2;
                for c in 0..COLORS_PER_PALETTE as usize {
                    let bank = &t.banks[n];
                    let color = bank.palette[c];
                    let size = 6;
                    canvas.draw_rectangle(
                        color_x,
                        y,
                        size,
                        size,
                        Color::new(color.r() * 36, color.g() * 36, color.b() * 36, color.a() * 36),
                    );
                    color_x += size + gap;
                }

                y += 8;
                let lines = 4;

                // Subpalettes
                y += 2;
                let sub_size_x = 6;
                let sub_size_y = 6;
                let frame_y = y;
                let mut frame_height = 0;
                y += 1;
                for l in 0..lines {
                    for c in 0..SUBPALETTE_COUNT as usize / lines {
                        let bank = &t.banks[n];
                        let subpal = bank.sub_palettes[(l * lines) + c];

                        for (i, index) in subpal.iter().enumerate() {
                            let color = bank.palette[index.0 as usize];
                            canvas.draw_rectangle(
                                x + (sub_size_x * i as i32) + (c as i32 * sub_size_x * 4),
                                y,
                                sub_size_x,
                                sub_size_y,
                                Color::new(
                                    color.r() * 36,
                                    color.g() * 36,
                                    color.b() * 36,
                                    color.a() * 36,
                                ),
                            );
                        }
                    }
                    y += 8;
                    frame_height += 8;
                }

                //frames
                canvas.draw_rectangle_lines(x, frame_y, debug_w, frame_height, Color::BLACK);
                for x in (x .. x + debug_w).step_by(sub_size_x as usize * COLORS_PER_TILE as usize) {
                    canvas.draw_line(x, frame_y, x, frame_y + frame_height, Color::BLACK);
                }

                // tiles
                y += 2;
                tile_debug_texture[n].update_texture(&pixel_data).unwrap();
                canvas.draw_texture_ex(
                    &tile_debug_texture[n],
                    Vector2::new(x as f32, y as f32),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
                y += 132;
            }
        }
    }
}
