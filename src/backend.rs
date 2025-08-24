//! Backend trait for abstracting rendering operations across different graphics libraries

use crate::{Tato, prelude::Dashboard};
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

    /// Present the rendered frame to the screen
    fn present<'a, const LEN:usize, T>(
        &mut self,
        tato: &'a Tato,
        dash: Option<&'a mut Dashboard<LEN>>,
        // arena: &'a mut Arena<LEN>,
        bg_banks: &[&'a T],
    ) where
        &'a T: Into<TilemapRef<'a>>;

    /// Check if the window should close
    fn should_close(&self) -> bool;

    // ---------------------- Drawing ----------------------

    /// Measure text dimensions for the given font size
    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32);

    // /// Draw a filled rectangle
    // fn draw_rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: RGBA32);

    // /// Draw text at the specified position
    // fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: RGBA32);

    // /// Draw a line between two points
    // fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32);

    // ---------------------- Texture Management ----------------------

    /// Create a new texture and return its ID
    fn create_texture(&mut self, width: i16, height: i16) -> TextureId;

    /// Update an existing texture with new pixel data
    fn update_texture(&mut self, id: TextureId, pixels: &[u8]);

    /// Draw a texture at the specified position with scaling and tint
    fn draw_texture(&mut self, id: TextureId, rect: Rect<i16>, tint: RGBA32);

    // ---------------------- Input ----------------------

    /// Get current mouse position
    fn get_mouse(&self) -> Vec2<i16>;

    /// Update gamepad/input state
    fn update_input(&mut self, pad: &mut AnaloguePad);

    // ---------------------- Window Info ----------------------

    /// Get screen dimensions
    fn get_screen_size(&self) -> Vec2<i16>;

    /// Set window title
    fn set_window_title(&mut self, title: &str);

    /// Set target FPS
    fn set_target_fps(&mut self, fps: u32);

    /// Set backend color where Tato pixels are transparent
    fn set_bg_color(&mut self, color: RGBA32);

    // ---------------------- State ----------------------

    /// Toggle debug mode and return new state
    fn toggle_debug(&mut self) -> bool;

    /// Check if debug mode is enabled
    fn debug_mode(&self) -> bool;
}
