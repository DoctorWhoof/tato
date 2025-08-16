use tato::backend::TextureId;
use tato::math::Rect;
use tato::video::RGBA32;

/// A drawing command that can be executed by any backend
#[derive(Clone, Debug)]
pub enum DrawOp {
    Rect { rect: Rect<i16>, color: RGBA32 },
    Line { x1: i16, y1: i16, x2: i16, y2: i16, color: RGBA32 },
    Texture { id: TextureId, x: i16, y: i16, scale: f32, tint: RGBA32 },
    Text { text: String, x: f32, y: f32, size: f32, color: RGBA32 },
}
