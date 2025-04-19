// use raylib::{ffi::KeyboardKey, prelude::RaylibDraw, RaylibHandle, RaylibThread, core::texture::Texture2D};
use crate::PIXEL_COUNT;
use raylib::prelude::*;
use tato::prelude::*;

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
}

pub fn copy_pixels_to_texture(
    vid: &VideoChip,
    thread: &RaylibThread,
    ray: &mut RaylibHandle,
    pixels: &mut [u8; PIXEL_COUNT],
    texture: &mut Texture2D,
) {
    // Copy from framebuffer to raylib texture
    for (color, coords) in vid.iter_pixels() {
        let i = ((coords.y as usize * 240) + coords.x as usize) * 4;
        pixels[i] = color.r;
        pixels[i + 1] = color.g;
        pixels[i + 2] = color.b;
        pixels[i + 3] = 255;
    }
    texture.update_texture(pixels).unwrap();

    // Calculate rect with correct aspect ratio with integer scaling
    let screen_width = ray.get_screen_width();
    let screen_height = ray.get_screen_height();

    let scale = (screen_height as f32 / vid.height() as f32).floor() as i32;
    let w = vid.width() as i32 * scale;
    let h = vid.height() as i32 * scale;
    let draw_rect_x = (screen_width - w) / 2;
    let draw_rect_y = (screen_height - h) / 2;

    // Present frame
    let mut canvas = ray.begin_drawing(thread);
    canvas.clear_background(Color::BLACK);
    canvas.draw_texture_ex(
        &texture,
        Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
        0.0,
        scale as f32,
        Color::WHITE,
    );
}
