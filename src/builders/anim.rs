use tato_video::*;

#[derive(Debug, Clone)]
pub(crate) struct AnimBuilder {
    pub name: String,
    // pub id: u8,
    pub frames: Vec<FrameBuilder>,
    pub fps: u8,
    pub columns: u8,
    pub rows: u8,
}

/// The smallest part of an animation, contains tiles indices up to ANIM_TILES_PER_FRAME.
#[derive(Debug, Clone)]
pub struct FrameBuilder {
    // pub cols: u8,
    // pub rows: u8,
    pub tiles: Vec<TileEntry>,
}

// impl FrameBuilder {
//     // pub fn from_slice(slice: &[Tile], cols: u8, rows: u8) -> FrameBuilder {
//     pub fn from_slice(slice: &[Tile]) -> FrameBuilder {
//         FrameBuilder {
//             // cols,
//             // rows,
//             tiles: slice.into(),
//         }
//     }
// }

// impl AnimBuilder {
// pub fn new(name: String, fps: u8, columns: u8) -> Self {
//     AnimBuilder {
//         name,
//         // id,
//         frames: Vec::new(),
//         fps,
//         columns,
//     }
// }

// pub fn push(&mut self, frame: FrameBuilder) {
//     if self.frames.len() == 255 {
//         panic!("Anim error: capacity of 255 frames exceeded.")
//     };
//     self.frames.push(frame);
// }

// pub fn id(&self) -> u8 {
//     self.id
// }
// }
