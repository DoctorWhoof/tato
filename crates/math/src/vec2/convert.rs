use super::*;

impl Vec2<i8> {
    pub fn to_f32(self) -> Vec2<f32> {
        Vec2 { x: self.x as f32, y: self.y as f32 }
    }
}

impl Vec2<f32> {
    // This seems sufficient to avoid seams between entities!
    pub fn to_i32(self) -> Vec2<i32> {
        Vec2 { x: self.x.floor() as i32, y: self.y.floor() as i32 }
    }
}