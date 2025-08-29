pub use raylib;
use raylib::prelude::*;
use std::{time::Instant, vec};
use tato::{
    Tato, arena::*, backend::Backend, dashboard::*, prelude::*, smooth_buffer::SmoothBuffer,
};

pub use tato;

const TILES_PER_ROW: i16 = 16;

#[inline]
fn rgba32_to_rl_color(color: RGBA32) -> Color {
    Color::new(color.r, color.g, color.b, color.a)
}

pub struct RaylibBackend {
    pub bg_color: Color,
    pub integer_scaling: bool,
    pub print_frame_time: bool,
    pub ray: RaylibHandle,
    pub canvas_rect: Option<Rect<i16>>,
    thread: RaylibThread,
    textures: Vec<Texture2D>,
    font: Font,
    draw_ops: Buffer<ArenaId<DrawOp, u32>, u32>,
    draw_ops_additional: Buffer<ArenaId<DrawOp, u32>, u32>,
    canvas_texture: TextureId,
    // Cached then passed to Dashboard later
    dash_args: DashArgs,
    pixels: Vec<u8>,
    buffer_iter_time: SmoothBuffer<120, f64>,
    buffer_canvas_time: SmoothBuffer<120, f64>,
}

/// Raylib specific implementation
impl RaylibBackend {
    pub fn new<const LEN: usize>(tato: &Tato, frame_arena: &mut Arena<LEN, u32>) -> Self {
        // Sizing
        let multiplier = 3;
        let w = tato.video.width() as i32;
        let h = tato.video.height() as i32;
        let total_panel_width = (PANEL_WIDTH * 4) + (MARGIN * 2);
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
            integer_scaling: true,
            print_frame_time: true,
            canvas_rect: None,
            ray,
            thread,
            // debug_pixels,
            textures: vec![],
            font,
            draw_ops: Buffer::new(frame_arena, 1000).unwrap(),
            draw_ops_additional: Buffer::default(),
            canvas_texture: 0,
            dash_args: DashArgs::default(),
            pixels: vec![0; w as usize * h as usize * 4],
            buffer_iter_time: SmoothBuffer::new(),
            buffer_canvas_time: SmoothBuffer::new(),
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
}

/// Main API, using Backend trait
impl Backend for RaylibBackend {
    // ---------------------- Core Rendering ----------------------

    fn clear(&mut self, color: RGBA32) {
        // This will be called in the render loop, storing for later use
        self.bg_color = rgba32_to_rl_color(color);
    }

    fn frame_start<const LEN: usize>(&mut self, frame_arena: &mut Arena<LEN, u32>) {
        self.draw_ops = Buffer::new(frame_arena, 1000).unwrap();
    }

    /// Finish canvas and GUI drawing, present to window
    fn frame_present<'a, const LEN: usize, T>(
        &mut self,
        frame_arena: &'a mut Arena<LEN, u32>,
        tato: &'a Tato,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>,
    {
        let time_profile = Instant::now();
        // let mut temp_texts = Arena::<32768, u32>::new();

        assert!(
            {
                let video_width = tato.video.width() as usize;
                let video_height = tato.video.height() as usize;
                video_width * video_height * 4
            } == self.pixels.len(),
        );

        // Copy pixels from video chip
        for (i, color) in tato.iter_pixels(bg_banks).enumerate() {
            let index = i * 4;
            self.pixels[index] = color.r;
            self.pixels[index + 1] = color.g;
            self.pixels[index + 2] = color.b;
            self.pixels[index + 3] = color.a;
        }
        self.buffer_iter_time.push(time_profile.elapsed().as_secs_f64());


        // Update main render texture and queue draw operation
        Self::update_texture_internal(
            &mut self.textures,
            &mut self.ray,
            &self.thread,
            self.canvas_texture,
            self.pixels.as_slice(),
        );

        // Draw canvas, if not in debug mode or dashboard not available
        if self.dash_args.display_debug {
            if let Some(canvas_rect) = self.canvas_rect {
                // Adjust canvas to fit rect
                let (rect, _scale) =
                    canvas_rect_and_scale(canvas_rect, tato.video.size(), false);
                // Queue drawing
                let op_id = frame_arena
                    .alloc(DrawOp::Texture {
                        id: self.canvas_texture,
                        rect,
                        tint: RGBA32::WHITE,
                    })
                    .unwrap();
                self.draw_ops.push(frame_arena, op_id).unwrap();
            }
        } else {
            let screen_size = self.get_screen_size();
            let screen_rect = Rect { x: 0, y: 0, w: screen_size.x, h: screen_size.y };
            let (canvas_rect, _scale) = canvas_rect_and_scale(screen_rect, tato.video.size(), true);
            let op_id = frame_arena
                .alloc(DrawOp::Texture {
                    id: self.canvas_texture,
                    rect: canvas_rect,
                    tint: RGBA32::WHITE,
                })
                .unwrap();
            self.draw_ops.push(frame_arena, op_id).unwrap();
        }

        self.dash_args = DashArgs {
            iter_time: self.buffer_iter_time.average() as f32,
            screen_size: self.get_screen_size(),
            canvas_size: tato.video.size(),
            mouse: self.get_mouse(),
            ..self.dash_args
        };

        // But if dashboard is available, queue GUI drawing
        // if let Some(dash) = dash {
        //     if self.display_debug() {
        //         if let Some(canvas_rect) = dash.canvas_rect() {
        //             // Adjust canvas to fit rect
        //             // let (rect, _scale) =
        //             //     canvas_rect_and_scale(canvas_rect, tato.video.size(), false);
        //             // Queue drawing
        //             self.draw_texture(self.canvas_texture, canvas_rect, RGBA32::WHITE);
        //         }

        //         self.dash_args = DashArgs {
        //             screen_size: self.get_screen_size(),
        //             canvas_size: tato.video.size(),
        //             mouse: self.get_mouse(),
        //             ..self.dash_args
        //         };

        //         // Push timing data before moving ops out of dashboard
        //         dash.push_text(&format!(
        //             "Pixel iter time: {:.1} ms", //
        //             self.buffer_iter_time.average() * 1000.0
        //         ));
        //         dash.push_text(&format!(
        //             "Canvas queue time: {:.1} ms",
        //             self.buffer_canvas_time.average() * 1000.0
        //         ));

        //         // Generate debug UI (this populates tile_pixels but doesn't update GPU textures)
        //         dash.render(tato, self.dash_args);

        //         // Copy tile pixels from dashboard to GPU textures
        //         for bank_index in 0..TILE_BANK_COUNT {
        //             // texture ID = bank_index
        //             if let Some(pixels) = dash.tile_pixels(bank_index) {
        //                 if !pixels.is_empty() {
        //                     Self::update_texture_internal(
        //                         &mut self.textures,
        //                         &mut self.ray,
        //                         &self.thread,
        //                         bank_index,
        //                         pixels,
        //                     );
        //                 }
        //             }
        //         }
        //     }

        //     for op in dash.draw_ops().unwrap() {
        //         match op {
        //             DrawOp::None => {},
        //             DrawOp::Text { text, x, y, size, color } => {
        //                 if let Some(text_str) = text.as_str(dash.temp_arena()) {
        //                     let new_text = Text::from_str(&mut temp_texts, text_str);
        //                     if let Ok(text) = new_text {
        //                         self.draw_ops.push(DrawOp::Text {
        //                             text,
        //                             x: *x,
        //                             y: *y,
        //                             size: *size,
        //                             color: *color,
        //                         })
        //                     }
        //                 }
        //             },
        //             _ => self.draw_ops.push(op.clone()),
        //         }
        //     }
        // }

        // Start canvas drawing
        let mut canvas = self.ray.begin_drawing(&self.thread);
        canvas.clear_background(self.bg_color);

        let mut process_draw_ops = |cmd: &DrawOp| match cmd {
            DrawOp::None => {},
            DrawOp::Rect { rect, color } => {
                canvas.draw_rectangle(
                    rect.x as i32,
                    rect.y as i32,
                    rect.w as i32,
                    rect.h as i32,
                    rgba32_to_rl_color(*color),
                );
            },
            DrawOp::Line { x1, y1, x2, y2, color } => {
                canvas.draw_line(
                    *x1 as i32,
                    *y1 as i32,
                    *x2 as i32,
                    *y2 as i32,
                    rgba32_to_rl_color(*color),
                );
            },
            DrawOp::Texture { id, rect, tint } => {
                if *id < self.textures.len() {
                    let w = self.textures[*id].width() as f32;
                    let scale = rect.w as f32 / w;
                    canvas.draw_texture_ex(
                        &self.textures[*id],
                        Vector2::new(rect.x as f32, rect.y as f32),
                        0.0,
                        scale,
                        rgba32_to_rl_color(*tint),
                    );
                }
            },
            DrawOp::Text { text, x, y, size, color } => {
                if let Some(text) = text.as_str(&frame_arena) {
                    canvas.draw_text_ex(
                        &self.font,
                        text,
                        Vector2::new(*x as f32, *y as f32),
                        *size,
                        1.0,
                        rgba32_to_rl_color(*color),
                    );
                }
            },
        };

        // Execute draw ops
        for id in self.draw_ops.drain(frame_arena) {
            let cmd = frame_arena.get(&id).unwrap();
            process_draw_ops(cmd);
        }

        for id in self.draw_ops_additional.drain(frame_arena) {
            let cmd = frame_arena.get(&id).unwrap();
            process_draw_ops(cmd);
        }

        // Time to queue all backed drawing, does not include actual render time,
        // which will happen when this function returns
        self.buffer_canvas_time.push(time_profile.elapsed().as_secs_f64());

        // TODO: This print exists for a silly reason: the game actually runs slower if I don't! :-0
        // CPU usage increases and Frame Update time increases if I don't print every frame. Super weird.
        // I believe it's related to Efficiency cores Vs. Performance ones.
        if self.print_frame_time {
            let time = self.buffer_canvas_time.average() + self.buffer_iter_time.average();
            println!(
                "Frame {} finished in {:.2} ms (max {} fps)",
                tato.video.frame_number(),
                time * 1000.0,
                (1.0 / time).floor()
            );
        }
    }

    fn should_close(&self) -> bool {
        self.ray.window_should_close()
    }

    // ---------------------- Drawing Primitives ----------------------

    // Any arena allocated Op (like Text) is, at this point, using the same "frame_arena",
    // and so can simply be copied without converting anything!
    fn set_additional_draw_ops(&mut self, draw_ops: Buffer<ArenaId<DrawOp, u32>, u32>) {
        self.draw_ops_additional = draw_ops
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

        // Dashboard toggle
        if ray.is_key_pressed(KEY_TAB) {
            self.dash_args.display_debug = !self.dash_args.display_debug;
        }

        if ray.is_key_pressed(KEY_GRAVE) {
            if self.dash_args.display_debug {
                self.dash_args.display_console = !self.dash_args.display_console;
            }
        }

        // Dashboard keys. Always reset to none, then set most recent one, if any
        self.dash_args.key = Key::None;
        if let Some(key) = ray.get_key_pressed() {
            match key {
                KEY_ENTER => {
                    self.dash_args.key = Key::Enter;
                },
                KEY_TAB => {
                    self.dash_args.key = Key::Tab;
                },
                KEY_MINUS | KEY_KP_SUBTRACT => {
                    self.dash_args.key = Key::Minus;
                },
                KEY_EQUAL | KEY_KP_ADD => {
                    self.dash_args.key = Key::Plus;
                },
                KEY_BACKSPACE => {
                    self.dash_args.key = Key::Backspace;
                },
                KEY_DELETE => {
                    self.dash_args.key = Key::Delete;
                },
                KEY_LEFT => {
                    self.dash_args.key = Key::Left;
                },
                KEY_RIGHT => {
                    self.dash_args.key = Key::Right;
                },
                KEY_UP => {
                    self.dash_args.key = Key::Up;
                },
                KEY_DOWN => {
                    self.dash_args.key = Key::Down;
                },
                KEY_GRAVE => {
                    // Do nothing, to avoid being used as shortcut and entering text
                },
                _ if (key as u32) >= 32 && (key as u32) < 127 => {
                    // Handle all printable ASCII characters (32-126)
                    match key as u32 {
                        k if k >= KEY_A as u32 && k <= KEY_Z as u32 => {
                            // Letters: apply shift for case
                            if ray.is_key_down(KEY_LEFT_SHIFT) || ray.is_key_down(KEY_RIGHT_SHIFT) {
                                self.dash_args.key = Key::Text(key as u8); // uppercase
                            } else {
                                self.dash_args.key = Key::Text(key as u8 + 32); // lowercase
                            }
                        },
                        k if k >= KEY_ZERO as u32 && k <= KEY_NINE as u32 => {
                            // Number row: apply shift for symbols
                            if ray.is_key_down(KEY_LEFT_SHIFT) || ray.is_key_down(KEY_RIGHT_SHIFT) {
                                // Shift+number gives symbols: )!@#$%^&*(
                                let symbols = b")!@#$%^&*(";
                                self.dash_args.key =
                                    Key::Text(symbols[(k - KEY_ZERO as u32) as usize]);
                            } else {
                                self.dash_args.key = Key::Text(b'0' + (k - KEY_ZERO as u32) as u8);
                            }
                        },
                        k if k >= KEY_KP_0 as u32 && k <= KEY_KP_9 as u32 => {
                            // Keypad numbers (no shift variants)
                            self.dash_args.key = Key::Text(b'0' + (k - KEY_KP_0 as u32) as u8);
                        },
                        _ => {
                            // All other printable characters
                            self.dash_args.key = Key::Text(key as u8);
                        },
                    }
                },
                _ => {},
            }
        };
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

    fn set_canvas_rect(&mut self, canvas_rect: Option<Rect<i16>>) {
        self.canvas_rect = canvas_rect;
    }

    fn get_dashboard_args(&self) -> Option<DashArgs> {
        Some(self.dash_args)
    }

    // ---------------------- Debug Features ----------------------
}
