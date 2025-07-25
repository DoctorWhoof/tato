// use tato_video::Tilemap;
// use tato_arena::Pool;
// use tato_arena::Pool;
// use tato_video::Cell;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct AnimID(pub u8);

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct StripID(pub u8);

// A collection of animation frames (slices of Cells)
// stored in Assets. Indices refer to tilemap entries.
#[derive(Debug, Default)]
pub(crate) struct StripEntry {
    pub start_index: u8,
    pub frame_count: u8
}

// Tiny metadata to control animation playback of frames (stored as indices)
pub struct Anim<const LEN: usize> {
    pub fps: u8,
    pub repeat: bool,
    pub frames: [u8; LEN],
}
