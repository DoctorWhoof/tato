//! Backend trait for abstracting rendering operations across different graphics libraries

use crate::{prelude::{DrawOp, Key}, Tato};
use tato_arena::{Arena, ArenaId, Buffer};
use tato_math::{Rect, Vec2};
use tato_pad::AnaloguePad;
use tato_video::{RGBA32, TilemapRef};

/// Texture identifier
pub type TextureId = usize;

/// Calculate position and scale for centered integer scaling with correct aspect ratio
pub fn canvas_rect_and_scale(
    screen_rect: Rect<i16>,
    video_size: Vec2<i16>,
    integer: bool,
) -> (Rect<i16>, f32) {
    // Calculate aspect ratios
    let screen_aspect = screen_rect.w as f32 / screen_rect.h as f32;
    let video_aspect = video_size.x as f32 / video_size.y as f32;

    // Obtain scale, respect aspect ratio
    let mut scale = if screen_aspect < video_aspect {
        // Screen is narrower than video, fit to width
        screen_rect.w as f32 / video_size.x as f32
    } else {
        // Screen is wider than video, fit to height
        screen_rect.h as f32 / video_size.y as f32
    };

    if integer {
        scale = scale.floor()
    }
    // Generate output
    let w = (video_size.x as f32 * scale) as i16;
    let h = (video_size.y as f32 * scale) as i16;
    let x = ((screen_rect.w - w) / 2) + screen_rect.x;
    let y = ((screen_rect.h - h) / 2) + screen_rect.y;
    (Rect { x, y, w, h }, scale)
}

/// Core backend trait for rendering operations
pub trait Backend {
    // ---------------------- Main Rendering ----------------------

    /// Clear the screen with the given color
    fn clear(&mut self, color: RGBA32);

    fn frame_start<const LEN: usize>(&mut self, frame_arena: &mut Arena<LEN>, pad:&mut AnaloguePad);

    /// Present the rendered frame to the screen
    fn frame_present<'a, const LEN: usize, T>(
        &mut self,
        arena: &'a mut Arena<LEN>,
        tato: &'a Tato,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>;

    /// Check if the window should close
    fn should_close(&self) -> bool;

    // ---------------------- Drawing ----------------------

    fn set_additional_draw_ops(&mut self, draw_ops: Buffer<ArenaId<DrawOp, u32>, u32>);

    /// Measure text dimensions for the given font size
    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32);

    /// Create a new texture and return its ID
    fn create_texture(&mut self, width: i16, height: i16) -> TextureId;

    /// Update an existing texture with new pixel data
    fn update_texture(&mut self, id: TextureId, pixels: &[u8]);

    // ---------------------- Input ----------------------

    /// Get current mouse position
    fn get_mouse(&self) -> Vec2<i16>;

    fn get_pressed_key(&self) -> Option<Key>;

    // ---------------------- Window Info ----------------------

    fn get_elapsed_time(&self) -> f32;

    /// Set window title
    fn set_window_title(&mut self, title: &str);

    /// Set target FPS
    fn set_target_fps(&mut self, fps: u32);

    /// Set backend color where Tato pixels are transparent
    fn set_bg_color(&mut self, color: RGBA32);

    /// Optional canvas texture rect, useful to draw the main canvas inside a GUI
    fn set_canvas_rect(&mut self, canvas_rect: Option<Rect<i16>>);

    /// Get screen dimensions
    fn get_screen_size(&self) -> Vec2<i16>;

    // ---------------------- Profiling ----------------------

    fn toggle_info_printing(&mut self);

    fn get_pixel_iter_elapsed_time(&self) -> f32;

    fn get_drawing_elapsed_time(&self) -> f32;

}
