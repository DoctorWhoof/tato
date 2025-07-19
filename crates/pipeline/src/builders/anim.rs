// use tato_video::*;

// use crate::MapBuilder;

use crate::MapBuilder;

#[derive(Debug, Clone)]
pub(crate) struct AnimBuilder {
    pub name: String,
    pub frames: Vec<MapBuilder>,
    pub fps: u8,
    // pub frames: Vec<MapBuilder>,
    // pub columns: u8,
    // pub rows: u8,
}

// /// The smallest part of an animation, contains tiles indices up to ANIM_TILES_PER_FRAME.
// #[derive(Debug, Clone)]
// pub struct FrameBuilder {
//     pub tiles: Vec<Cell>,
// }
