use std::vec;

pub use raylib;
use raylib::prelude::*;
use tato::{Tato, prelude::*, smooth_buffer::SmoothBuffer};
use tato::backend::Backend;

pub use tato;

// pub struct RaylibBackend<const PIXEL_COUNT: usize> {
pub struct RaylibBackend {
    pub bg_color: Color,
    pub ray: RaylibHandle,
    pub display_debug: bool,
    pub display_debug_scale: i32,
    thread: RaylibThread,
    pixels: Vec<u8>,
    debug_pixels: Vec<Vec<u8>>,
    render_texture: Texture2D,
    debug_texture: Vec<Texture2D>,
    font: Font,
    iter_time_buffer: SmoothBuffer<300, f64>,
}

impl RaylibBackend {
    pub fn new(tato: &Tato) -> Self {
        let w = tato.video.width() as i32;
        let h = tato.video.height() as i32;
        let (mut ray, thread) = raylib::init()
            .log_level(TraceLogLevel::LOG_WARNING)
            .size(w * 4, h * 3)
            .title("Tato Demo")
            .vsync()
            .resizable()
            .build();

        // Config raylib
        ray.set_target_fps(tato.target_fps as u32);
        unsafe {
            raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
            // Disable ESC to close window. "Cmd + Q" still works!
            raylib::ffi::SetExitKey(raylib::ffi::KeyboardKey::KEY_NULL as i32);
        }

        // Embed Font file
        let font_data = include_bytes!("font.ttf");
        let font = ray.load_font_from_memory(&thread, ".ttf", font_data, 32, None).unwrap();

        // Create texture for rendering
        let pixels = vec![0u8; w as usize * h as usize * 4];
        let render_texture = {
            let render_image = Image::gen_image_color(w, h, Color::BLACK);
            ray.load_texture_from_image(&thread, &render_image).unwrap()
        };

        // Pre-populate textures and pixel buffers
        let mut debug_texture = vec![];
        let mut debug_pixels = vec![];
        let tiles_per_row = (TILE_COUNT as f64).sqrt().ceil() as usize;
        let tiles_w = tiles_per_row * TILE_SIZE as usize;
        for _bank in &tato.banks {
            // Allocate for maximum possible tiles (TILE_COUNT) instead of current tile count
            // since tiles may be loaded after backend initialization
            let max_rows = (TILE_COUNT / tiles_per_row) + 1;
            let tiles_h = max_rows * TILE_SIZE as usize;
            let debug_image = Image::gen_image_color(tiles_w as i32, tiles_h as i32, Color::BLACK);
            debug_texture.push(ray.load_texture_from_image(&thread, &debug_image).unwrap());
            debug_pixels.push(vec![0u8; tiles_w * tiles_h * 4]);
        }

        // Build struct
        Self {
            bg_color: Color::new(32, 32, 32, 255),
            ray,
            display_debug: true,
            display_debug_scale: 1,
            thread,
            pixels,
            debug_pixels,
            render_texture,
            debug_texture,
            font,
            iter_time_buffer: SmoothBuffer::pre_filled(1.0 / 120.0),
        }
    }

    pub fn update_gamepad(&self, pad: &mut AnaloguePad) {
        // Use the trait implementation
        self.update_input(pad);
    }

    pub fn render<'a, T>(&mut self, t: &'a Tato, bg_banks: &[&'a T])
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        // ---------------------- Copy from framebuffer to raylib texture ----------------------

        let time = std::time::Instant::now();
        for (color, coords) in t.iter_pixels(bg_banks) {
            let i = ((coords.y as usize * t.video.width() as usize) + coords.x as usize) * 4;
            self.pixels[i] = color.r;
            self.pixels[i + 1] = color.g;
            self.pixels[i + 2] = color.b;
            self.pixels[i + 3] = color.a;
        }
        self.iter_time_buffer.push(time.elapsed().as_secs_f64() * 1000.0);

        self.render_texture.update_texture(&self.pixels).unwrap();

        // Calculate rect with correct aspect ratio with integer scaling
        let screen_width = self.ray.get_screen_width();
        let screen_height = self.ray.get_screen_height();

        let scale = (screen_height as f32 / t.video.height() as f32).floor() as i32;
        let w = t.video.width() as i32 * scale;
        let h = t.video.height() as i32 * scale;
        let draw_rect_x = (screen_width - w) / 2;
        let draw_rect_y = (screen_height - h) / 2;

        // Present pixels
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.clear_background(self.bg_color);
        canvas.draw_texture_ex(
            &self.render_texture,
            Vector2::new(draw_rect_x as f32, draw_rect_y as f32),
            0.0,
            scale as f32,
            Color::WHITE,
        );
    }
}



fn rgba32_to_rl_color(color: RGBA32) -> Color {
    Color::new(color.r, color.g, color.b, color.a)
}

impl Backend for RaylibBackend {
    // ---------------------- Core Rendering ----------------------
    
    fn clear(&mut self, color: RGBA32) {
        // This will be called in the render loop, storing for later use
        self.bg_color = rgba32_to_rl_color(color);
    }
    
    fn present(&mut self) {
        // Raylib handles presentation automatically in the render loop
        // This is a no-op for raylib
    }
    
    fn should_close(&self) -> bool {
        self.ray.window_should_close()
    }

    // ---------------------- Main Texture Operations ----------------------
    
    fn update_main_texture(&mut self, pixels: &[u8], _width: u32, _height: u32) {
        let copy_len = pixels.len().min(self.pixels.len());
        for i in 0..copy_len {
            self.pixels[i] = pixels[i];
        }
        self.render_texture.update_texture(&self.pixels).unwrap();
    }
    
    fn draw_main_texture(&mut self, rect: Rect<i16>, scale: i32) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.clear_background(self.bg_color);
        canvas.draw_texture_ex(
            &self.render_texture,
            Vector2::new(rect.x as f32, rect.y as f32),
            0.0,
            scale as f32,
            Color::WHITE,
        );
    }

    // ---------------------- Drawing Primitives ----------------------
    
    fn draw_rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: RGBA32) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.draw_rectangle(x as i32, y as i32, w as i32, h as i32, rgba32_to_rl_color(color));
    }
    
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: RGBA32) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.draw_text_ex(
            &self.font,
            text,
            Vector2::new(x, y),
            font_size,
            1.0,
            rgba32_to_rl_color(color),
        );
    }
    
    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
        let size = self.font.measure_text(text, font_size, 1.0);
        (size.x, size.y)
    }
    
    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.draw_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32, rgba32_to_rl_color(color));
    }

    // ---------------------- Texture Management ----------------------
    
    fn create_texture(&mut self, width: i16, height: i16) -> TextureId {
        let image = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let texture = self.ray.load_texture_from_image(&self.thread, &image).unwrap();
        self.debug_texture.push(texture);
        self.debug_texture.len() - 1
    }
    
    fn update_texture(&mut self, id: TextureId, pixels: &[u8]) {
        if id < self.debug_texture.len() && id < self.debug_pixels.len() {
            if pixels.len() <= self.debug_pixels[id].len() {
                self.debug_pixels[id][0..pixels.len()].copy_from_slice(pixels);
                self.debug_texture[id].update_texture(&self.debug_pixels[id]).unwrap();
            }
        }
    }
    
    fn draw_texture(&mut self, id: TextureId, x: f32, y: f32, scale: f32, tint: RGBA32) {
        if id < self.debug_texture.len() {
            let mut canvas = self.ray.begin_drawing(&self.thread);
            canvas.draw_texture_ex(
                &self.debug_texture[id],
                Vector2::new(x, y),
                0.0,
                scale,
                rgba32_to_rl_color(tint),
            );
        }
    }

    // ---------------------- Input ----------------------
    
    fn mouse_pos(&self) -> (i16, i16) {
        (self.ray.get_mouse_x() as i16, self.ray.get_mouse_y() as i16)
    }
    
    fn update_input(&self, pad: &mut AnaloguePad) {
        // Copy existing update_gamepad logic
        pad.copy_current_to_previous_state();

        // Handle keyboard input
        pad.set_button(Button::Left, self.ray.is_key_down(KeyboardKey::KEY_LEFT));
        pad.set_button(Button::Right, self.ray.is_key_down(KeyboardKey::KEY_RIGHT));
        pad.set_button(Button::Up, self.ray.is_key_down(KeyboardKey::KEY_UP));
        pad.set_button(Button::Down, self.ray.is_key_down(KeyboardKey::KEY_DOWN));
        pad.set_button(Button::Menu, self.ray.is_key_down(KeyboardKey::KEY_ESCAPE));
        pad.set_button(Button::Start, self.ray.is_key_down(KeyboardKey::KEY_ENTER));
        pad.set_button(Button::A, self.ray.is_key_down(KeyboardKey::KEY_Z));
        pad.set_button(Button::LeftShoulder, self.ray.is_key_down(KeyboardKey::KEY_ONE));
    }

    // ---------------------- Window Info ----------------------
    
    fn screen_size(&self) -> (i16, i16) {
        (self.ray.get_screen_width() as i16, self.ray.get_screen_height() as i16)
    }
    
    fn set_window_title(&mut self, title: &str) {
        self.ray.set_window_title(&self.thread, title);
    }
    
    fn set_target_fps(&mut self, fps: u32) {
        self.ray.set_target_fps(fps);
    }

    // ---------------------- Debug Features ----------------------
    
    fn toggle_debug(&mut self) -> bool {
        self.display_debug = !self.display_debug;
        self.display_debug
    }
    
    fn set_debug_scale(&mut self, scale: i32) {
        self.display_debug_scale = scale.max(1);
    }
    
    fn get_debug_scale(&self) -> i32 {
        self.display_debug_scale
    }
    
    fn debug_mode(&self) -> bool {
        self.display_debug
    }
}
