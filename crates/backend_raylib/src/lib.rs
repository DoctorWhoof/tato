use std::vec;

pub use raylib;
use raylib::prelude::*;
use tato::backend::Backend;
use tato::dashboard::DrawOp;
use tato::{Tato, prelude::*, smooth_buffer::SmoothBuffer};

pub use tato;

const TILES_PER_ROW: i16 = 16;

#[inline]
fn rgba32_to_rl_color(color: RGBA32) -> Color {
    Color::new(color.r, color.g, color.b, color.a)
}

pub struct RaylibBackend {
    bg_color: Color,
    pub ray: RaylibHandle,
    pub display_debug: bool,
    thread: RaylibThread,
    textures: Vec<Texture2D>,
    canvas_texture: TextureId,
    font: Font,
    iter_time_buffer: SmoothBuffer<300, f64>,
    draw_ops: Vec<DrawOp>,
    dash: Dashboard, // Performs Debug UI drawing, stores debug pixels for tile banks
}

/// Raylib specific implementation
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

        // Build struct
        let mut result = Self {
            bg_color: Color::new(32, 32, 32, 255),
            ray,
            display_debug: true,
            thread,
            // debug_pixels,
            textures: vec![],
            canvas_texture: 0,
            font,
            iter_time_buffer: SmoothBuffer::pre_filled(1.0 / 120.0),
            draw_ops: Vec::new(),
            dash: Dashboard::new(),
        };

        let size = TILES_PER_ROW as i16 * TILE_SIZE as i16;
        for _ in 0..TILE_BANK_COUNT {
            // Each texture ID is the same as the bank
            result.create_texture(size, size);
        }

        // Main render texture (ID 0)
        let id = result.create_texture(tato.video.width() as i16, tato.video.height() as i16);
        result.canvas_texture = id;

        result
    }

    /// Copy from framebuffer to raylib texture
    pub fn render_canvas<'a, T>(&mut self, t: &'a Tato, bg_banks: &[&'a T])
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        let time = std::time::Instant::now();
        let video_width = t.video.width() as usize;
        let video_height = t.video.height() as usize;
        let expected_pixel_count = video_width * video_height;

        let pixels: Vec<u8> = t
            .iter_pixels(bg_banks)
            .take(expected_pixel_count)
            .flat_map(|(color, _)| [color.r, color.g, color.b, color.a])
            .collect();

        // Update main render texture and queue draw operation
        self.update_texture(self.canvas_texture, &pixels);

        self.iter_time_buffer.push(time.elapsed().as_secs_f64() * 1000.0);

        // Calculate and queue main texture draw
        let screen_size = self.get_screen_size();
        let (pos, scale) = canvas_position_and_scale(
            screen_size,
            Vec2::new(t.video.width() as i16, t.video.height() as i16),
        );
        self.draw_texture(self.canvas_texture, pos.x, pos.y, scale, RGBA32::WHITE);
        self.dash.canvas_scale = scale;
        self.dash.canvas_offset = pos;
    }

    /// Process "Dashboard" draw ops
    pub fn render_dashboard<'a>(&mut self, t: &'a Tato) {
        if !self.debug_mode() {
            return;
        }

        let screen_size = self.get_screen_size();
        let mouse = self.get_mouse();

        // Generate debug UI (this populates tile_pixels but doesn't update GPU textures)
        self.dash.render(screen_size, mouse, t);

        // Update GPU textures with the generated pixel data
        let source_pixels = self.dash.tile_pixels.clone();
        for bank_index in 0..TILE_BANK_COUNT {
            if let Some(ref pixels) = source_pixels.get(bank_index) {
                if !pixels.is_empty() {
                    self.update_texture(bank_index, pixels); // Update texture ID = bank_index
                }
            }
        }

        for op in &self.dash.ops {
            self.draw_ops.push(op.clone());
        }
    }
}

/// Main API, using Backend trait
impl Backend for RaylibBackend {
    // ---------------------- Core Rendering ----------------------

    fn clear(&mut self, color: RGBA32) {
        // This will be called in the render loop, storing for later use
        self.bg_color = rgba32_to_rl_color(color);
    }

    /// Finish canvas and GUI drawing, present to window
    fn present(&mut self) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.clear_background(self.bg_color);
        // Execute draw ops
        for cmd in self.draw_ops.drain(..) {
            match cmd {
                DrawOp::Rect { x, y, w, h, color } => {
                    canvas.draw_rectangle(
                        x as i32,
                        y as i32,
                        w as i32,
                        h as i32,
                        rgba32_to_rl_color(color),
                    );
                },
                DrawOp::Line { x1, y1, x2, y2, color } => {
                    canvas.draw_line(
                        x1 as i32,
                        y1 as i32,
                        x2 as i32,
                        y2 as i32,
                        rgba32_to_rl_color(color),
                    );
                },
                DrawOp::Texture { id, x, y, scale, tint } => {
                    if id < self.textures.len() {
                        canvas.draw_texture_ex(
                            &self.textures[id],
                            Vector2::new(x as f32, y as f32),
                            0.0,
                            scale,
                            rgba32_to_rl_color(tint),
                        );
                    }
                },
                DrawOp::Text { text, x, y, size, color } => {
                    canvas.draw_text_ex(
                        &self.font,
                        &text,
                        Vector2::new(x, y),
                        size,
                        1.0,
                        rgba32_to_rl_color(color),
                    );
                },
            }
        }
    }

    fn should_close(&self) -> bool {
        self.ray.window_should_close()
    }

    // ---------------------- Drawing Primitives ----------------------

    fn draw_rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: RGBA32) {
        self.draw_ops.push(DrawOp::Rect { x, y, w, h, color });
    }

    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: RGBA32) {
        self.draw_ops.push(DrawOp::Text { text: text.to_string(), x, y, size: font_size, color });
    }

    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32) {
        self.draw_ops.push(DrawOp::Line { x1, y1, x2, y2, color });
    }

    fn draw_texture(&mut self, id: TextureId, x: i16, y: i16, scale: f32, tint: RGBA32) {
        self.draw_ops.push(DrawOp::Texture { id, x, y, scale, tint });
    }

    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
        let size = self.font.measure_text(text, font_size, 1.0);
        (size.x, size.y)
    }
    // ---------------------- Texture Management ----------------------

    fn create_texture(&mut self, width: i16, height: i16) -> TextureId {
        let image = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let texture = self.ray.load_texture_from_image(&self.thread, &image).unwrap();
        self.textures.push(texture);
        self.textures.len() - 1
    }

    fn update_texture(&mut self, id: TextureId, pixels: &[u8]) {
        if id < self.textures.len() {
            let len = self.textures[id].width as usize * self.textures[id].height as usize * 4;
            if pixels.len() != len {
                // resize texture to match
                let total_pixels = pixels.len() / 4;
                let new_w = TILES_PER_ROW as i32 * TILE_SIZE as i32;
                let new_h = (total_pixels / new_w as usize) as i32;
                let image = Image::gen_image_color(new_w, new_h, Color::BLACK);
                let texture = self.ray.load_texture_from_image(&self.thread, &image).unwrap();
                self.textures[id] = texture;
            }
            self.textures[id].update_texture(&pixels).unwrap();
        }
    }

    // ---------------------- Input ----------------------

    fn get_mouse(&self) -> Vec2<i16> {
        Vec2::new(self.ray.get_mouse_x() as i16, self.ray.get_mouse_y() as i16)
    }

    fn update_input(&mut self, pad: &mut AnaloguePad) {
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

        // Dashboard
        if self.ray.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.display_debug = !self.display_debug;
        }
        if self.ray.is_key_pressed(KeyboardKey::KEY_EQUAL) {
            self.dash.gui_scale += 1;
        }
        if self.ray.is_key_pressed(KeyboardKey::KEY_MINUS) {
            if self.dash.gui_scale > 1 {
                self.dash.gui_scale -= 1;
            }
        }
    }

    // ---------------------- Window Info ----------------------

    fn get_screen_size(&self) -> Vec2<i16> {
        let width:i32 = self.ray.get_screen_width();
        let height:i32 = self.ray.get_screen_height();
        Vec2::new(width as i16, height as i16)
    }

    fn set_window_title(&mut self, title: &str) {
        self.ray.set_window_title(&self.thread, title);
    }

    fn set_target_fps(&mut self, fps: u32) {
        self.ray.set_target_fps(fps);
    }

    fn set_bg_color(&mut self, color: RGBA32) {
        self.bg_color = rgba32_to_rl_color(color)
    }
    // ---------------------- Debug Features ----------------------

    fn toggle_debug(&mut self) -> bool {
        self.display_debug = !self.display_debug;
        self.display_debug
    }

    fn debug_mode(&self) -> bool {
        self.display_debug
    }
}
