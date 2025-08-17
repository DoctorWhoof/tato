use tato_arena::Text;
use tato_math::Rect;
use tato_video::RGBA32;

/// A drawing command that can be executed by any backend
#[derive(Clone, Debug, Default)]
pub enum DrawOp {
    #[default]
    None,
    Rect {
        rect: Rect<i16>,
        color: RGBA32,
    },
    Line {
        x1: i16,
        y1: i16,
        x2: i16,
        y2: i16,
        color: RGBA32,
    },
    Texture {
        id: usize,
        x: i16,
        y: i16,
        scale: f32,
        tint: RGBA32,
    },
    Text {
        text: Text,
        x: i16,
        y: i16,
        size: f32,
        color: RGBA32,
    },
}
