use tato_math::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct DashArgs {
    pub screen_size: Vec2<i16>,
    pub canvas_size: Vec2<i16>,
    pub mouse: Vec2<i16>,
    // pub canvas_rect: Rect<i16>,
    // pub canvas_scale: f32,
    // pub canvas_pos: Vec2<i16>,
    pub gui_scale: f32,
    pub console_display: bool,
    pub console_char: Option<u8>
}

impl Default for DashArgs {
    fn default() -> Self {
        Self {
            screen_size: Vec2 { x: 800, y: 600 },
            canvas_size: Vec2 { x:320, y:240 },
            mouse: Vec2 { x: 0, y: 0 },
            // canvas_rect: Rect { x: 0, y: 0, w: 320, h: 240 },
            // canvas_scale: 1.0,
            // canvas_pos: Vec2 { x: 0, y: 0 },
            gui_scale: 2.0,
            console_display: false,
            console_char: None
        }
    }
}
