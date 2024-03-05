use spud::*;


pub struct Player {
    pub id: EntityID,
    pub health: i16,
    pub score: u16,
    pub vel: Vec2<f32>
}