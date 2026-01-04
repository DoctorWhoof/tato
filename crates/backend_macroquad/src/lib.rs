//! Macroquad backend for Tato game development framework
//!
//! This crate provides a Macroquad-based implementation of the Tato Backend trait,
//! enabling cross-platform rendering with hardware acceleration.

pub use macroquad;
use macroquad::prelude::*;
use std::{time::Instant, vec};
use tato::{arena::*, avgbuffer::AvgBuffer, backend::Backend, dashboard::*, prelude::*, Tato};

pub use tato;

const TILES_PER_ROW: i16 = 16;

/// Create a window configuration with VSync enabled and proper settings for Tato games
///
/// This function provides a pre-configured `Conf` struct optimized for pixel art games,
/// with VSync enabled to prevent screen tearing and proper scaling settings.
///
/// # Arguments
///
/// * `title` - Window title
/// * `width` - Window width in pixels
/// * `height` - Window height in pixels
///
/// # Returns
///
/// A `Conf` struct ready to use with `#[macroquad::main(window_conf)]`
///
/// # Example
///
/// ```rust,no_run
/// use tato_macroquad::*;
///
/// fn window_conf() -> Conf {
///     tato_window_conf("My Game", 900, 540)
/// }
///
/// #[macroquad::main(window_conf)]
/// async fn main() {
///     // Your game code here
/// }
/// ```
pub fn tato_window_conf(title: &str, width: i32, height: i32) -> Conf {
    Conf {
        window_title: title.to_owned(),
        window_width: width,
        window_height: height,
        window_resizable: true,
        high_dpi: true,
        // VSync is enabled by default in Macroquad
        // Additional platform-specific optimizations
        ..Default::default()
    }
}

/// Convert RGBA32 color to Macroquad Color (0.0-1.0 range)
#[inline]
fn rgba32_to_mq_color(color: RGBA32) -> Color {
    Color::new(
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
        color.a as f32 / 255.0,
    )
}

/// Macroquad-based backend for Tato games
///
/// This backend provides cross-platform rendering using Macroquad's graphics capabilities.
/// It supports desktop platforms (Windows, macOS, Linux), web (WebAssembly), and mobile
/// platforms (Android, iOS).
///
/// # Usage
///
/// ```rust,no_run
/// use tato::prelude::*;
/// use tato_macroquad::*;
/// use macroquad::prelude::*;
///
/// #[macroquad::main("My Game")]
/// async fn main() -> TatoResult<()> {
///     let tato = Tato::new(240, 180, 60);
///     let mut backend = MquadBackend::new(&tato).await;
///
///     // Game loop here...
///     Ok(())
/// }
/// ```
pub struct MquadBackend {
    /// Background clear color
    pub bg_color: Color,
    /// Whether to use integer scaling for pixel-perfect rendering
    pub integer_scaling: bool,
    /// Whether to print frame timing information for debugging
    pub print_frame_time: bool,
    /// Optional canvas rendering rectangle for GUI integration
    pub canvas_rect: Option<tato::prelude::Rect<i16>>,
    textures: Vec<Texture2D>,
    font: Font,
    draw_ops: Buffer<ArenaId<DrawOp>>,
    draw_ops_additional: Buffer<ArenaId<DrawOp>>,
    canvas_texture: TextureId,
    pixels: Vec<u8>,
    buffer_iter_time: AvgBuffer<120, f32>,
    buffer_canvas_time: AvgBuffer<120, f32>,
    pressed_key: Option<Key>,
    allow_game_input: bool,
    target_fps: u32,
}

impl MquadBackend {
    /// Create a new Macroquad backend
    ///
    /// This is an async function that initializes the backend with the given Tato instance.
    /// The backend will create textures for tile banks and load the embedded font.
    ///
    /// # Arguments
    ///
    /// * `tato` - The Tato instance to create the backend for
    ///
    /// # Returns
    ///
    /// A new `MquadBackend` ready for rendering
    pub async fn new(tato: &Tato) -> Self {
        // Sizing
        let multiplier = 3;
        let w = tato.video.width() as i32;
        let h = tato.video.height() as i32;
        let total_panel_width = (PANEL_WIDTH * 4) + (MARGIN * 2);
        let adjusted_w = total_panel_width as i32 + (w * multiplier);

        // Configure window with VSync enabled
        request_new_screen_size(adjusted_w as f32, (h * multiplier) as f32);

        // Load embedded font
        let font_data = include_bytes!("font.ttf");
        let font = load_ttf_font_from_bytes(font_data).unwrap();

        // Build struct
        let mut result = Self {
            bg_color: Color::new(16.0 / 255.0, 16.0 / 255.0, 16.0 / 255.0, 1.0),
            integer_scaling: true,
            print_frame_time: false,
            canvas_rect: None,
            textures: vec![],
            font,
            draw_ops: Buffer::default(),
            draw_ops_additional: Buffer::default(),
            canvas_texture: 0,
            pixels: vec![0; w as usize * h as usize * 4],
            buffer_iter_time: AvgBuffer::new(),
            buffer_canvas_time: AvgBuffer::new(),
            pressed_key: None,
            allow_game_input: true,
            target_fps: tato.target_fps as u32,
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

    /// Internal method to update texture data
    #[inline(always)]
    fn update_texture_internal(textures: &mut [Texture2D], id: TextureId, pixels: &[u8]) {
        if id < textures.len() {
            let texture_pixel_count =
                textures[id].width() as usize * textures[id].height() as usize * 4;

            // resize texture to match
            if pixels.len() != texture_pixel_count {
                println!(
                    "Resizing texture, Source pixels: {}, dest pixels: {}",
                    pixels.len(),
                    texture_pixel_count
                );
                // Calculate number of tiles (each tile is 8x8 with 4 bytes per pixel)
                let total_tiles = pixels.len() / (TILE_SIZE as usize * TILE_SIZE as usize * 4);
                let complete_rows = total_tiles / TILES_PER_ROW as usize;
                let remaining_tiles = total_tiles % TILES_PER_ROW as usize;

                let new_w = TILES_PER_ROW as u16 * TILE_SIZE as u16;
                let complete_lines = complete_rows * TILE_SIZE as usize;
                let incomplete_lines = if remaining_tiles > 0 { TILE_SIZE as usize } else { 0 };
                let new_h = (complete_lines + incomplete_lines) as u16;

                let image = Image::gen_image_color(new_w, new_h, BLACK);
                let texture = Texture2D::from_image(&image);
                // texture.set_filter(FilterMode::Nearest);
                textures[id] = texture;
                println!("Backend texture {} resized", id);
            }

            // Convert pixels to Image for update
            let width = textures[id].width() as u16;
            let height = textures[id].height() as u16;

            let image = Image { width, height, bytes: pixels.to_vec() };
            textures[id].update(&image);
            textures[id].set_filter(FilterMode::Nearest);
        }
    }
}

impl Backend for MquadBackend {
    // ---------------------- Core Rendering ----------------------

    fn clear(&mut self, color: RGBA32) {
        // This will be called in the render loop, storing for later use
        self.bg_color = rgba32_to_mq_color(color);
    }

    fn set_game_input(&mut self, state: bool) {
        self.allow_game_input = state;
    }

    fn frame_start<A>(&mut self, frame_arena: &mut A, pad: &mut AnaloguePad)
    where
        A: ArenaOps<u32, ()>,
    {
        self.draw_ops = Buffer::new(frame_arena, 1000).unwrap();
        self.pressed_key = None;
        self.canvas_rect = None;

        // Gamepad input
        pad.copy_current_to_previous_state();
        if self.allow_game_input {
            pad.set_button(Button::Left, is_key_down(KeyCode::Left));
            pad.set_button(Button::Right, is_key_down(KeyCode::Right));
            pad.set_button(Button::Up, is_key_down(KeyCode::Up));
            pad.set_button(Button::Down, is_key_down(KeyCode::Down));
            pad.set_button(Button::Menu, is_key_down(KeyCode::Escape));
            pad.set_button(Button::Start, is_key_down(KeyCode::Enter));
            pad.set_button(Button::A, is_key_down(KeyCode::Z));
            pad.set_button(Button::B, is_key_down(KeyCode::X));
            pad.set_button(Button::X, is_key_down(KeyCode::A));
            pad.set_button(Button::Y, is_key_down(KeyCode::S));
            pad.set_button(Button::LeftShoulder, is_key_down(KeyCode::Q));
            pad.set_button(Button::RightShoulder, is_key_down(KeyCode::W));
        } else {
            // Clears any key being pressed when game input was disabled
            pad.clear();
        }

        // Dashboard keys ignore self.allow_game_input
        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Enter => {
                    self.pressed_key = Some(Key::Enter);
                },
                KeyCode::Tab => {
                    self.pressed_key = Some(Key::Tab);
                },
                KeyCode::Minus | KeyCode::KpSubtract => {
                    self.pressed_key = Some(Key::Minus);
                },
                KeyCode::Equal | KeyCode::KpAdd => {
                    self.pressed_key = Some(Key::Plus);
                },
                KeyCode::Backspace => {
                    self.pressed_key = Some(Key::Backspace);
                },
                KeyCode::Delete => {
                    self.pressed_key = Some(Key::Delete);
                },
                KeyCode::Left => {
                    self.pressed_key = Some(Key::Left);
                },
                KeyCode::Right => {
                    self.pressed_key = Some(Key::Right);
                },
                KeyCode::Up => {
                    self.pressed_key = Some(Key::Up);
                },
                KeyCode::Down => {
                    self.pressed_key = Some(Key::Down);
                },
                KeyCode::GraveAccent => {
                    self.pressed_key = Some(Key::Grave);
                },
                // Regular text
                _ if (key as u32) >= 32
                    && (key as u32) < 127
                    && !is_key_down(KeyCode::LeftSuper)
                    && !is_key_down(KeyCode::RightSuper)
                    && !is_key_down(KeyCode::LeftControl)
                    && !is_key_down(KeyCode::RightControl) =>
                {
                    // Handle letters A-Z
                    if matches!(
                        key,
                        KeyCode::A
                            | KeyCode::B
                            | KeyCode::C
                            | KeyCode::D
                            | KeyCode::E
                            | KeyCode::F
                            | KeyCode::G
                            | KeyCode::H
                            | KeyCode::I
                            | KeyCode::J
                            | KeyCode::K
                            | KeyCode::L
                            | KeyCode::M
                            | KeyCode::N
                            | KeyCode::O
                            | KeyCode::P
                            | KeyCode::Q
                            | KeyCode::R
                            | KeyCode::S
                            | KeyCode::T
                            | KeyCode::U
                            | KeyCode::V
                            | KeyCode::W
                            | KeyCode::X
                            | KeyCode::Y
                            | KeyCode::Z
                    ) {
                        // Letters: apply shift for case
                        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                            self.pressed_key = Some(Key::Text(key as u8)); // uppercase
                        } else {
                            self.pressed_key = Some(Key::Text(key as u8 + 32)); // lowercase
                        }
                    }
                    // Handle number keys 0-9
                    else if matches!(
                        key,
                        KeyCode::Key0
                            | KeyCode::Key1
                            | KeyCode::Key2
                            | KeyCode::Key3
                            | KeyCode::Key4
                            | KeyCode::Key5
                            | KeyCode::Key6
                            | KeyCode::Key7
                            | KeyCode::Key8
                            | KeyCode::Key9
                    ) {
                        // Number row: apply shift for symbols
                        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                            // Shift+number gives symbols: )!@#$%^&*(
                            let symbols = b")!@#$%^&*(";
                            let index = (key as u32 - KeyCode::Key0 as u32) as usize;
                            self.pressed_key = Some(Key::Text(symbols[index]));
                        } else {
                            self.pressed_key =
                                Some(Key::Text(b'0' + (key as u32 - KeyCode::Key0 as u32) as u8));
                        }
                    }
                    // Handle keypad numbers
                    else if matches!(
                        key,
                        KeyCode::Kp0
                            | KeyCode::Kp1
                            | KeyCode::Kp2
                            | KeyCode::Kp3
                            | KeyCode::Kp4
                            | KeyCode::Kp5
                            | KeyCode::Kp6
                            | KeyCode::Kp7
                            | KeyCode::Kp8
                            | KeyCode::Kp9
                    ) {
                        // Keypad numbers (no shift variants)
                        self.pressed_key =
                            Some(Key::Text(b'0' + (key as u32 - KeyCode::Kp0 as u32) as u8));
                    }
                    // Handle other characters
                    else {
                        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                            match key {
                                KeyCode::Slash => self.pressed_key = Some(Key::Text(b'?')),
                                KeyCode::Period => self.pressed_key = Some(Key::Text(b'>')),
                                KeyCode::Comma => self.pressed_key = Some(Key::Text(b'<')),
                                _ => self.pressed_key = Some(Key::Text(key as u8)),
                            }
                        } else {
                            self.pressed_key = Some(Key::Text(key as u8));
                        }
                    }
                },
                _ => {},
            }
        };
    }

    /// Finish canvas and GUI drawing, present to window
    fn frame_present<'a, A, T>(
        &mut self,
        frame_arena: &'a mut A,
        tato: &'a Tato,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>,
        A: ArenaOps<u32, ()>,
    {
        let time_iter = Instant::now();

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
        self.buffer_iter_time.push(time_iter.elapsed().as_secs_f32());

        // Update main render texture and queue draw operation
        let time_queue = Instant::now();
        Self::update_texture_internal(
            &mut self.textures,
            self.canvas_texture,
            self.pixels.as_slice(),
        );

        // Draw canvas, using canvas_rect if available
        if let Some(canvas_rect) = self.canvas_rect {
            // Adjust canvas to fit rect
            let (rect, _scale) = canvas_rect_and_scale(canvas_rect, tato.video.size(), false);
            // Queue drawing
            let op_id = frame_arena
                .alloc(DrawOp::Texture { id: self.canvas_texture, rect, tint: RGBA32::WHITE })
                .unwrap();
            self.draw_ops.push(frame_arena, op_id).unwrap();
        } else {
            let screen_size = self.get_screen_size();
            let screen_rect =
                tato::prelude::Rect { x: 0, y: 0, w: screen_size.x, h: screen_size.y };
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

        // Start drawing
        clear_background(self.bg_color);

        let process_draw_ops = |cmd: &DrawOp| match cmd {
            DrawOp::None => {},
            DrawOp::Rect { rect, color } => {
                draw_rectangle(
                    rect.x as f32,
                    rect.y as f32,
                    rect.w as f32,
                    rect.h as f32,
                    rgba32_to_mq_color(*color),
                );
            },
            DrawOp::Line { x1, y1, x2, y2, color } => {
                draw_line(
                    *x1 as f32,
                    *y1 as f32,
                    *x2 as f32,
                    *y2 as f32,
                    1.0,
                    rgba32_to_mq_color(*color),
                );
            },
            DrawOp::Texture { id, rect, tint } => {
                if *id < self.textures.len() {
                    draw_texture_ex(
                        &self.textures[*id],
                        rect.x as f32,
                        rect.y as f32,
                        rgba32_to_mq_color(*tint),
                        DrawTextureParams {
                            dest_size: Some(macroquad::math::Vec2::new(
                                rect.w as f32,
                                rect.h as f32,
                            )),
                            ..Default::default()
                        },
                    );
                }
            },
            DrawOp::Text { text, x, y, size, color } => {
                if let Ok(text) = text.as_str(frame_arena) {
                    draw_text_ex(
                        text,
                        *x as f32,
                        *y as f32,
                        TextParams {
                            font: Some(&self.font),
                            font_size: *size as u16,
                            color: rgba32_to_mq_color(*color),
                            ..Default::default()
                        },
                    );
                }
            },
        };

        // Execute draw ops
        if let Ok(slice) = self.draw_ops.as_slice(frame_arena) {
            for ids in slice {
                if let Ok(cmd) = frame_arena.get(*ids) {
                    process_draw_ops(cmd);
                }
            }
        }

        if let Ok(slice) = self.draw_ops_additional.as_slice(frame_arena) {
            for ids in slice {
                if let Ok(cmd) = frame_arena.get(*ids) {
                    process_draw_ops(cmd);
                }
            }
        }

        // Time to queue all backed drawing, does not include actual render time,
        // which will happen when this function returns
        self.buffer_canvas_time.push(time_queue.elapsed().as_secs_f32());

        // This print exists for a silly reason: the game actually runs slower if I don't! :-0
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
        // In macroquad, window close is typically handled differently
        // Return false here and handle close in the main loop
        false
    }

    // ---------------------- Drawing Primitives ----------------------

    // Any arena allocated Op (like Text) is, at this point, using the same "frame_arena",
    // and so can simply be copied without converting anything!
    fn set_additional_draw_ops(&mut self, draw_ops: Buffer<ArenaId<DrawOp>>) {
        self.draw_ops_additional = draw_ops
    }

    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32) {
        let dimensions = measure_text(text, Some(&self.font), font_size as u16, 1.0);
        (dimensions.width, dimensions.height)
    }

    // ---------------------- Texture Management ----------------------

    fn create_texture(&mut self, width: i16, height: i16) -> TextureId {
        let image = Image::gen_image_color(width as u16, height as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);
        self.textures.push(texture);
        self.textures.len() - 1
    }

    fn update_texture(&mut self, id: TextureId, pixels: &[u8]) {
        Self::update_texture_internal(&mut self.textures, id, pixels);
    }

    // ---------------------- Input ----------------------

    fn get_mouse(&self) -> tato::prelude::Vec2<i16> {
        let (x, y) = mouse_position();
        tato::prelude::Vec2::new(x as i16, y as i16)
    }

    // ---------------------- Window Info ----------------------

    fn get_pressed_key(&self) -> Option<Key> {
        self.pressed_key
    }

    fn get_elapsed_time(&self) -> f32 {
        get_frame_time()
    }

    fn set_window_title(&mut self, _title: &str) {
        // Macroquad doesn't have runtime window title setting in the same way
        // This would need to be handled differently or ignored
    }

    fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps;
        // Macroquad doesn't have a direct set_target_fps equivalent
        // FPS is typically controlled via the main loop
    }

    fn set_bg_color(&mut self, color: RGBA32) {
        self.bg_color = rgba32_to_mq_color(color)
    }

    fn set_canvas_rect(&mut self, canvas_rect: Option<tato::prelude::Rect<i16>>) {
        self.canvas_rect = canvas_rect;
    }

    fn get_screen_size(&self) -> tato::prelude::Vec2<i16> {
        tato::prelude::Vec2::new(screen_width() as i16, screen_height() as i16)
    }

    fn get_pixel_iter_elapsed_time(&self) -> f32 {
        self.buffer_iter_time.average() as f32
    }

    fn get_drawing_elapsed_time(&self) -> f32 {
        self.buffer_canvas_time.average() as f32
    }

    fn toggle_info_printing(&mut self) {
        self.print_frame_time = !self.print_frame_time
    }

    // ---------------------- Debug Features ----------------------
}
