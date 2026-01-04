use std::time::Instant;
use tato::{Tato, arena::*, avgbuffer::AvgBuffer, backend::Backend, dashboard::*, prelude::*};

pub use tato;

pub struct DummyBackend {
    pub print_frame_time: bool,
    buffer_iter_time: AvgBuffer<120, f32>,
    last_print_time: Instant,
    pressed_key: Option<Key>,
    frame_count: u64,
}

impl DummyBackend {
    pub fn new(_tato: &Tato) -> Self {
        Self {
            print_frame_time: true,
            buffer_iter_time: AvgBuffer::new(),
            last_print_time: Instant::now(),
            pressed_key: None,
            frame_count: 0,
        }
    }
}

impl Backend for DummyBackend {
    // ---------------------- Core Rendering ----------------------

    fn clear(&mut self, _color: RGBA32) {
        // Do nothing - dummy backend doesn't actually display
    }

    fn set_game_input(&mut self, _state: bool) {
        // Do nothing - dummy backend doesn't handle input
    }

    fn frame_start<A>(&mut self, _frame_arena: &mut A, pad: &mut AnaloguePad)
    where
        A: ArenaOps<u32, ()>,
    {
        self.pressed_key = None;
        // Clear pad input since we're not handling any
        pad.clear();
    }

    fn frame_present<'a, A, T>(
        &mut self,
        _frame_arena: &'a mut A,
        tato: &'a Tato,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>,
        A: ArenaOps<u32, ()>,
    {
        let time_iter = Instant::now();

        // Iterate through all pixels just like Raylib backend does, but don't store them
        let mut pixel_count = 0;
        for _color in tato.iter_pixels(bg_banks) {
            // Silly task just to be busy and not get optimized out
            pixel_count += 1;
        }

        self.buffer_iter_time.push(time_iter.elapsed().as_secs_f64());
        self.frame_count += 1;

        // Print performance info every second
        if self.print_frame_time && self.last_print_time.elapsed().as_secs() >= 1 {
            let avg_time = self.buffer_iter_time.average();
            println!(
                "Dummy Backend - Frame {} processed {} pixels in {:.2} ms (max {} fps)",
                self.frame_count,
                pixel_count,
                avg_time * 1000.0,
                (1.0 / avg_time).floor()
            );
            self.last_print_time = Instant::now();
        }
    }

    fn should_close(&self) -> bool {
        // Never close - dummy backend runs indefinitely for testing
        false
    }

    // ---------------------- Drawing Primitives ----------------------

    fn set_additional_draw_ops(&mut self, _draw_ops: Buffer<ArenaId<DrawOp>>) {
        // Do nothing - dummy backend doesn't draw
    }

    fn measure_text(&self, text: &str, _font_size: f32) -> (f32, f32) {
        // Return approximate text size (8 pixels per character, 12 pixels height)
        (text.len() as f32 * 8.0, 12.0)
    }

    // ---------------------- Texture Management ----------------------

    fn create_texture(&mut self, _width: i16, _height: i16) -> TextureId {
        // Return dummy texture ID
        0
    }

    fn update_texture(&mut self, _id: TextureId, _pixels: &[u8]) {
        // Do nothing - dummy backend doesn't handle textures
    }

    // ---------------------- Input ----------------------

    fn get_mouse(&self) -> Vec2<i16> {
        Vec2::new(0, 0)
    }

    fn get_pressed_key(&self) -> Option<Key> {
        self.pressed_key
    }

    // ---------------------- Window Info ----------------------

    fn get_elapsed_time(&self) -> f32 {
        // Return a fixed delta time for consistent testing
        1.0 / 60.0 // 60 FPS
    }

    fn set_window_title(&mut self, _title: &str) {
        // Do nothing - dummy backend has no window
    }

    fn set_target_fps(&mut self, _fps: u32) {
        // Do nothing - dummy backend doesn't control FPS
    }

    fn set_bg_color(&mut self, _color: RGBA32) {
        // Do nothing - dummy backend doesn't display
    }

    fn set_canvas_rect(&mut self, _canvas_rect: Option<Rect<i16>>) {
        // Do nothing - dummy backend has no canvas
    }

    fn get_screen_size(&self) -> Vec2<i16> {
        // Return dummy screen size
        Vec2::new(800, 600)
    }

    fn get_pixel_iter_elapsed_time(&self) -> f32 {
        self.buffer_iter_time.average() as f32
    }

    fn get_drawing_elapsed_time(&self) -> f32 {
        // Return 0 since we don't actually draw anything
        0.0
    }

    fn toggle_info_printing(&mut self) {
        self.print_frame_time = !self.print_frame_time;
    }

    // ---------------------- Debug Features ----------------------
}
