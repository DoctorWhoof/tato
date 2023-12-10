use crate::*;

pub struct CharState {
    pub action: Action,
    // pub direction: Direction,
    // pub modifier: Modifier,
    pub up_tile: TileProbe,
    pub down_tile: TileProbe,
    pub left_tile: TileProbe,
    pub right_tile: TileProbe,
    pub speed: u8,           //Pixels per second
    pub anim_idle: AnimID,
    pub anim_run: AnimID,
    pub vel: Vec2<f32>
}

#[derive(PartialEq)]
pub enum Action {
    Idle,
    Moving,
    // Jumping,
    // Falling,
    // Interacting,
    // Dead
}

// pub enum Direction {
//     Left,
//     Right,
//     // Up,
//     // Down
// }


// pub enum Modifier {
//     Land,
//     Water
// }


impl Default for CharState {
    fn default() -> Self {
        Self {
            action:Action::Idle,
            // direction: Direction::Right,
            // modifier:Modifier::Land,
            speed: 120,
            anim_idle: AnimID(0),
            anim_run: AnimID(0),
            vel: Vec2 { x: 0.0, y: 0.0 },
            up_tile: TileProbe::default(),
            down_tile: TileProbe::default(),
            left_tile: TileProbe::default(),
            right_tile: TileProbe::default(),
            
        }
    }
}
