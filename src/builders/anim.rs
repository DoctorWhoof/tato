use tato_video::*;

#[derive(Debug, Clone)]
pub(crate) struct AnimBuilder {
    pub name: String,
    pub frames: Vec<FrameBuilder>,
    pub fps: u8,
    pub columns: u8,
    pub rows: u8,
}

/// The smallest part of an animation, contains tiles indices up to ANIM_TILES_PER_FRAME.
#[derive(Debug, Clone)]
pub struct FrameBuilder {
    pub tiles: Vec<Cell>,
}
