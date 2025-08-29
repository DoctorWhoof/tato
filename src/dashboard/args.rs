use tato_math::Vec2;
use super::Key;

#[derive(Debug, Clone, Copy)]
pub struct DashArgs {
    pub iter_time: f32,
    pub screen_size: Vec2<i16>,
    pub canvas_size: Vec2<i16>,
    pub mouse: Vec2<i16>,
    pub key: Key,
    pub display_console:bool,
    pub display_debug:bool,
}

impl Default for DashArgs {
    fn default() -> Self {
        Self {
            iter_time: 0.0,
            screen_size: Vec2 { x: 800, y: 600 },
            canvas_size: Vec2 { x:320, y:240 },
            mouse: Vec2 { x: 0, y: 0 },
            key: Key::None,
            display_console: false,
            display_debug: false,
        }
    }
}
