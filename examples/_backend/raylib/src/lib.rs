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
    // Copy from framebuffer to raylib texture
    let time = std::time::Instant::now();

    // let tiles = &t.tile_refs();
    // let tilemaps = &t.map_refs();
    let banks = t.get_video_banks();
    for (color, coords) in t.video.iter_pixels(&banks) {
        let i = ((coords.y as usize * t.video.width() as usize) + coords.x as usize) * 4;
        pixels[i] = color.r;
        pixels[i + 1] = color.g;
        pixels[i + 2] = color.b;
        pixels[i + 3] = color.a;
    }
    texture.update_texture(pixels).unwrap();
    t.update_time_acc.push(time.elapsed().as_secs_f64() * 1000.0);
    // if t.video.frame_count() % 60 == 0 {
        // println!("iter time: {:.2} ms", time.elapsed().as_secs_f64() * 1000.0);
        println!("iter time: {:.2} ms", t.update_time_acc.average());
    // }

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
    canvas.clear_background(Color::new(48, 32, 24, 255));
    canvas.draw_texture_ex(
        &texture,
        Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
        0.0,
        scale as f32,
        Color::WHITE,
    );
}
