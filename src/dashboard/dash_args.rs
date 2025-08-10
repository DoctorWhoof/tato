use tato_math::Vec2;


#[derive(Debug, Clone, Copy)]
pub struct DashArgs {
    pub screen_size: Vec2<i16>,
    pub mouse: Vec2<i16>,
    pub canvas_scale: f32,
    pub canvas_pos: Vec2<i16>,
    pub gui_scale: f32,
}

impl Default for DashArgs {
    fn default() -> Self {
        Self {
            screen_size: Vec2 { x: 800, y: 600 },
            mouse: Vec2 { x: 0, y: 0 },
            canvas_scale: 1.0,
            canvas_pos: Vec2 { x: 0, y: 0 },
            gui_scale: 1.0,
        }
    }
}
