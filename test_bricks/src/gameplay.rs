use tato::{EntityID, Vec2};
use crate:: Input;

pub struct Paddle {
    pub id: EntityID,
    pub vel: Vec2<f32>,
    pub input: Input,
}

#[derive(Default)]
pub struct Puck {
    pub id: EntityID,
    pub vel: Vec2<f32>,
    pub initial_pos: Vec2<f32>,
    // pub extra_vel: Vec2<f32>
    // pub prev_angles: [i32; 2],
    // pub prev_pos: [Vec2<f32>; 3],
    // pub prev_head:usize
}
