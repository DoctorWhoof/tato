pub use raylib;
use raylib::prelude::*;
use std::{time::Instant, vec};
use tato::{
    FRAME_ARENA_LEN, Tato, arena::*, backend::Backend, dashboard::*, prelude::*,
    smooth_buffer::SmoothBuffer,
};

pub use tato;

const TILES_PER_ROW: i16 = 16;

#[inline]
fn rgba32_to_rl_color(color: RGBA32) -> Color {
    Color::new(color.r, color.g, color.b, color.a)
}

pub struct RaylibBackend {
    pub ray: RaylibHandle,
    pub display_debug: bool,
    pub bg_color: Color,
    pub print_frame_time: bool,
    thread: RaylibThread,
    textures: Vec<Texture2D>,
    font: Font,
    draw_ops: Vec<DrawOp>,
    canvas_texture: TextureId,
    // Cached then passed to Dashboard later
    dash_args: DashArgs,
    pixels: Vec<u8>,
    time_profile: Instant,
    buffer_iter_time: SmoothBuffer<120, f64>,
    buffer_canvas_time: SmoothBuffer<120, f64>,
    frame_number: usize,
}

/// Raylib specific implementation
impl RaylibBackend {
    pub fn new(tato: &Tato) -> Self {
        // Sizing
        let multiplier = 3;
        let w = tato.video.width() as i32;
        let h = tato.video.height() as i32;
        let total_panel_width = (Dashboard::PANEL_WIDTH * 4) + (Dashboard::MARGIN * 4);
        let adjusted_w = total_panel_width as i32 + (w * multiplier);

        // Init Raylib
        let (mut ray, thread) = raylib::init()
            .log_level(TraceLogLevel::LOG_WARNING)
            .size(adjusted_w, h * multiplier)
            .title("Tato Demo")
            .vsync()
            .resizable()
            .build();

        // Config additional raylib options
        ray.set_target_fps(tato.target_fps as u32);
        unsafe {
            raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_VSYNC_HINT as u32);
            raylib::ffi::SetConfigFlags(raylib::ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
            // Disable ESC to close window. "Cmd + Q" still works!
            raylib::ffi::SetExitKey(raylib::ffi::KeyboardKey::KEY_NULL as i32);
        }

        // Embed Font file
        let font_data = include_bytes!("font.ttf");
        let font = ray.load_font_from_memory(&thread, ".ttf", font_data, 32, None).unwrap();

        // Build struct
        let mut result = Self {
            bg_color: Color::new(16, 16, 16, 255),
            ray,
            display_debug: true,
            print_frame_time: true,
            thread,
            // debug_pixels,
            textures: vec![],
            font,
            draw_ops: Vec::new(),
            canvas_texture: 0,
            dash_args: DashArgs::default(),
            pixels: vec![0; w as usize * h as usize * 4],
            time_profile: std::time::Instant::now(),
            buffer_iter_time: SmoothBuffer::new(),
            buffer_canvas_time: SmoothBuffer::new(),
            frame_number: 0,
        };

        let size = TILES_PER_ROW as i16 * TILE_SIZE as i16;
        for _ in 0..TILE_BANK_COUNT {
            // Each texture ID is the same as the bank
            result.create_texture(size, size);
        }

        // Main render texture
        let id = result.create_texture(tato.video.width() as i16, tato.video.height() as i16);
        result.canvas_texture = id;

        result
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
            // resize texture to match
            if pixels.len() != texture_pixel_count {
                println!("Souce pixels: {}, dest pixels: {}", pixels.len(), texture_pixel_count);
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
                println!("Backend texture {} resized", id);
            }
            textures[id].update_texture(&pixels).unwrap();
        }
    }

    /// Copy from framebuffer to raylib texture
    pub fn render_canvas<'a, T>(&mut self, t: &'a Tato, bg_banks: &[&'a T])
    where
        &'a T: Into<TilemapRef<'a>>,
    {
        self.frame_number = t.video.frame_number();
        self.time_profile = Instant::now();
        let video_width = t.video.width() as usize;
        let video_height = t.video.height() as usize;
        debug_assert!({ video_width * video_height * 4 } == self.pixels.len(),);

        for (i, color) in t.iter_pixels(bg_banks).enumerate() {
            let index = i * 4;
            self.pixels[index] = color.r;
            self.pixels[index + 1] = color.g;
            self.pixels[index + 2] = color.b;
            self.pixels[index + 3] = color.a;
        }
        self.buffer_iter_time.push(self.time_profile.elapsed().as_secs_f64());

        // Will be used across functions, that's why it's a field
        // TODO: Proper profiling...
        self.time_profile = Instant::now();

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
    pub fn render_dashboard<'a>(
        &mut self,
        t: &'a Tato,
        dash: &mut Dashboard,
        arena: &mut Arena<FRAME_ARENA_LEN>,
    ) {
        if !self.debug_mode() {
            return;
        }

        // Push timing data before moving ops out of dashboard
        dash.add_text(
            &format!(
                "iter time: {:.1} ms", //
                self.buffer_iter_time.average() * 1000.0
            ),
            arena,
        );
        dash.add_text(
            &format!("canvas time: {:.1} ms", self.buffer_canvas_time.average() * 1000.0),
            arena,
        );

        // Generate debug UI (this populates tile_pixels but doesn't update GPU textures)
        dash.render(t, arena, self.dash_args);

        // Copy tile pixels from dashboard to GPU textures
        for bank_index in 0..TILE_BANK_COUNT {
            // texture ID = bank_index
            if let Some(pixels) = dash.tile_pixels.get(bank_index) {
                if !pixels.is_empty() {
                    Self::update_texture_internal(
                        &mut self.textures,
                        &mut self.ray,
                        &self.thread,
                        bank_index,
                        pixels.as_slice(&dash.pixel_arena).unwrap(),
                    );
                }
            }
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
    fn present(&mut self, _tato: &Tato, dash: Option<&Dashboard>, arena: &Arena<FRAME_ARENA_LEN>) {
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.clear_background(self.bg_color);

        let mut process_draw_op = |cmd: DrawOp| match cmd {
            DrawOp::None => {},
            DrawOp::Rect { rect, color } => {
                canvas.draw_rectangle(
                    rect.x as i32,
                    rect.y as i32,
                    rect.w as i32,
                    rect.h as i32,
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
                let text = text.as_str(arena).unwrap();
                canvas.draw_text_ex(
                    &self.font,
                    text,
                    Vector2::new(x as f32, y as f32),
                    size,
                    1.0,
                    rgba32_to_rl_color(color),
                );
            },
        };

        // Execute draw ops
        for cmd in self.draw_ops.drain(..) {
            process_draw_op(cmd)
        }
        if let Some(dash) = dash {
            for id in dash.ops.items(arena).unwrap() {
                let op = arena.get(id).unwrap();
                process_draw_op(op.clone());
            }
        }

        // Time to queue all backed drawing, does not include actual render time,
        // which will happen when this function returns
        self.buffer_canvas_time.push(self.time_profile.elapsed().as_secs_f64());

        // TODO: This print exists for a silly reason: the game actually runs slower if I don't! :-0
        // CPU usage increases and Frame Update time increases if I don't print every frame. Super weird.
        // I believe it's related to Efficiency cores Vs. Performance ones.
        if self.print_frame_time {
            let time = self.buffer_canvas_time.average() + self.buffer_iter_time.average();
            println!(
                "Frame {} finished in {:.2} ms (max {} fps)",
                self.frame_number,
                time * 1000.0,
                (1.0 / time).floor()
            );
        }
    }

    fn should_close(&self) -> bool {
        self.ray.window_should_close()
    }

    // ---------------------- Drawing Primitives ----------------------

    // fn draw_rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: RGBA32) {
    //     self.draw_ops.push(DrawOp::Rect { rect: Rect { x, y, w, h }, color });
    // }

    // fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: RGBA32) {
    //     self.draw_ops.push(DrawOp::Text { text: text.to_string(), x, y, size: font_size, color });
    // }

    // fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32) {
    //     self.draw_ops.push(DrawOp::Line { x1, y1, x2, y2, color });
    // }

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
