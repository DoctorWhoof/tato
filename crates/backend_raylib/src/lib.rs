use std::vec;

pub use raylib;
use raylib::prelude::*;
use tato_dashboard::*;
use tato::{Tato, prelude::*, backend::Backend};

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
    font: Font,
    draw_ops: Vec<DrawOp>,
    canvas_texture: TextureId,
    // Cached then passed to Dashboard later
    dash_args: DashArgs,
    pixels: Vec<u8>,
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
            font,
            draw_ops: Vec::new(),
            canvas_texture: 0,
            dash_args: DashArgs::default(),
            pixels: vec![0; w as usize * h as usize * 4],
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
        debug_assert!(
            {
                let video_width = t.video.width() as usize;
                let video_height = t.video.height() as usize;
                video_width * video_height * 4
            } == self.pixels.len(),
        );

        for (i, (color, _)) in t.iter_pixels(bg_banks).enumerate() {
            let base_idx = i * 4;
            if base_idx + 3 < self.pixels.len() {
                self.pixels[base_idx] = color.r;
                self.pixels[base_idx + 1] = color.g;
                self.pixels[base_idx + 2] = color.b;
                self.pixels[base_idx + 3] = color.a;
            }
        }

        // Update main render texture and queue draw operation
        Self::update_texture_internal(
            &mut self.textures,
            &mut self.ray,
            &self.thread,
            self.canvas_texture,
            self.pixels.as_slice(),
        );

        // Calculate and queue main texture draw
        let screen_size = self.get_screen_size();
        let (pos, scale) = canvas_position_and_scale(
            screen_size,
            Vec2::new(t.video.width() as i16, t.video.height() as i16),
        );
        self.draw_texture(self.canvas_texture, pos.x, pos.y, scale, RGBA32::WHITE);
        self.dash_args = DashArgs {
            screen_size,
            mouse: self.get_mouse(),
            canvas_scale: scale,
            canvas_pos: pos,
            ..self.dash_args
        }
    }

    /// Process "Dashboard" draw ops
    pub fn render_dashboard<'a>(&mut self, dash: &mut Dashboard, t: &'a Tato) {
        if !self.debug_mode() {
            return;
        }
        // Generate debug UI (this populates tile_pixels but doesn't update GPU textures)
        dash.render(t, self.dash_args);
        // Update GPU textures with the generated pixel data
        for bank_index in 0..TILE_BANK_COUNT {
            // texture ID = bank_index
            if let Some(pixels) = dash.tile_pixels.get(bank_index) {
                if !pixels.is_empty() {
                    Self::update_texture_internal(
                        &mut self.textures,
                        &mut self.ray,
                        &self.thread,
                        bank_index,
                        pixels.as_slice(),
                    );
                }
            }
        }
        // Transfer ops from dashboard to internal buffer
        for op in dash.ops.drain(..) {
            self.draw_ops.push(op);
        }
    }

    #[inline(always)]
    fn update_texture_internal(
        textures: &mut [Texture2D],
        ray: &mut RaylibHandle,
        thread: &RaylibThread,
        id: TextureId,
        pixels: &[u8],
    ) {
        if id < textures.len() {
            let texture_pixel_count =
                textures[id].width as usize * textures[id].height as usize * 4;
            if pixels.len() != texture_pixel_count {
                // resize texture to match
                println!("Backend tile texture {} resized", id);

                // Calculate number of tiles (each tile is 8x8 with 4 bytes per pixel)
                let total_tiles = pixels.len() / (TILE_SIZE as usize * TILE_SIZE as usize * 4);
                let complete_rows = total_tiles / TILES_PER_ROW as usize;
                let remaining_tiles = total_tiles % TILES_PER_ROW as usize;

                let new_w = TILES_PER_ROW as i32 * TILE_SIZE as i32;
                let complete_lines = complete_rows * TILE_SIZE as usize;
                let incomplete_lines = if remaining_tiles > 0 { TILE_SIZE as usize } else { 0 };
                let new_h = (complete_lines + incomplete_lines) as i32;
                let image = Image::gen_image_color(new_w, new_h, Color::BLACK);
                let texture = ray.load_texture_from_image(thread, &image).unwrap();
                textures[id] = texture;
            }
            textures[id].update_texture(&pixels).unwrap();
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
        Self::update_texture_internal(&mut self.textures, &mut self.ray, &self.thread, id, pixels);
    }

    // ---------------------- Input ----------------------

    fn get_mouse(&self) -> Vec2<i16> {
        Vec2::new(self.ray.get_mouse_x() as i16, self.ray.get_mouse_y() as i16)
    }

    fn update_input(&mut self, pad: &mut AnaloguePad) {
        use KeyboardKey::*;
        let ray = &mut self.ray;

        // Copy existing update_gamepad logic
        pad.copy_current_to_previous_state();

        // Handle keyboard input
        pad.set_button(Button::Left, ray.is_key_down(KEY_LEFT));
        pad.set_button(Button::Right, ray.is_key_down(KEY_RIGHT));
        pad.set_button(Button::Up, ray.is_key_down(KEY_UP));
        pad.set_button(Button::Down, ray.is_key_down(KEY_DOWN));
        pad.set_button(Button::Menu, ray.is_key_down(KEY_ESCAPE));
        pad.set_button(Button::Start, ray.is_key_down(KEY_ENTER));
        pad.set_button(Button::A, ray.is_key_down(KEY_Z));
        pad.set_button(Button::LeftShoulder, ray.is_key_down(KEY_ONE));

        // Dashboard
        if ray.is_key_pressed(KEY_TAB) {
            self.display_debug = !self.display_debug;
        }
        if ray.is_key_pressed(KEY_EQUAL) {
            self.dash_args.gui_scale += 1.0;
        }
        if ray.is_key_pressed(KEY_MINUS) {
            if self.dash_args.gui_scale > 1.0 {
                self.dash_args.gui_scale -= 1.0;
            }
        }
    }

    // ---------------------- Window Info ----------------------

    fn get_screen_size(&self) -> Vec2<i16> {
        let width: i32 = self.ray.get_screen_width();
        let height: i32 = self.ray.get_screen_height();
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
