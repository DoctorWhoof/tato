//! Backend trait for abstracting rendering operations across different graphics libraries

use tato_pad::AnaloguePad;
use tato_video::RGBA32;
use tato_math::Rect;

/// Core backend trait for rendering operations
pub trait Backend {
    // ---------------------- Core Rendering ----------------------
    
    /// Clear the screen with the given color
    fn clear(&mut self, color: RGBA32);
    
    /// Present the rendered frame to the screen
    fn present(&mut self);
    
    /// Check if the window should close
    fn should_close(&self) -> bool;

    // ---------------------- Main Texture Operations ----------------------
    
    /// Update the main rendering texture with pixel data
    fn update_main_texture(&mut self, pixels: &[u8], width: u32, height: u32);
    
    /// Draw the main texture at the specified rectangle with scaling
    fn draw_main_texture(&mut self, rect: Rect<i16>, scale: i32);

    // ---------------------- Drawing Primitives ----------------------
    
    /// Draw a filled rectangle
    fn draw_rect(&mut self, x: i16, y: i16, w: i16, h: i16, color: RGBA32);
    
    /// Draw text at the specified position
    fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: RGBA32);
    
    /// Measure text dimensions for the given font size
    fn measure_text(&self, text: &str, font_size: f32) -> (f32, f32);
    
    /// Draw a line between two points
    fn draw_line(&mut self, x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32);

    // ---------------------- Texture Management ----------------------
    
    /// Create a new texture and return its ID
    fn create_texture(&mut self, width: i16, height: i16) -> TextureId;
    
    /// Update an existing texture with new pixel data
    fn update_texture(&mut self, id: TextureId, pixels: &[u8]);
    
    /// Draw a texture at the specified position with scaling and tint
    fn draw_texture(&mut self, id: TextureId, x: f32, y: f32, scale: f32, tint: RGBA32);

    // ---------------------- Input ----------------------
    
    /// Get current mouse position
    fn mouse_pos(&self) -> (i16, i16);
    
    /// Update gamepad/input state
    fn update_input(&self, pad: &mut AnaloguePad);

    // ---------------------- Window Info ----------------------
    
    /// Get screen dimensions
    fn screen_size(&self) -> (i16, i16);
    
    /// Set window title
    fn set_window_title(&mut self, title: &str);
    
    /// Set target FPS
    fn set_target_fps(&mut self, fps: u32);

    // ---------------------- Debug Features ----------------------
    
    /// Toggle debug mode and return new state
    fn toggle_debug(&mut self) -> bool;
    
    /// Set debug display scale
    fn set_debug_scale(&mut self, scale: i32);
    
    /// Get current debug scale
    fn get_debug_scale(&self) -> i32;
    
    /// Check if debug mode is enabled
    fn debug_mode(&self) -> bool;
}

// ---------------------- Backend-Agnostic Types ----------------------

/// Texture identifier
pub type TextureId = usize;

/// Font identifier  
pub type FontId = usize;

