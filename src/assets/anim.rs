use tato_arena::Pool;

/// A reference to a frame strip associated with a tileset.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct StripID(pub u8);

/// A collection of animation frames (slices of Cells)
/// stored in Assets. Indices refer to tilemap entries.
#[derive(Debug, Default)]
pub(crate) struct StripEntry {
    pub start_index: u8,
    pub frame_count: u8
}

/// A Reference to an animation object.
/// 
/// AnimID(0) represents "no animation" and is the default value.
/// Valid animation IDs start from 1.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Default)]
pub struct AnimID(pub u8);

/// Tiny metadata to control animation playback of frames already
/// stored under Assets.
/// TODO: Needs a way to be generated FROM a tileset's frame strip, with
/// some sort of validation.
pub struct Anim<const LEN: usize> {
    pub fps: u8,
    pub repeat: bool,
    pub frames: [u8; LEN],
    pub strip_id: StripID
}

#[derive(Debug, Clone, Default)]
pub struct AnimEntry {
    pub frames: Pool<u8, u16>,
    pub fps: u8,
    pub repeat: bool,
    pub strip_id: StripID
}

// /// A reference to an animation stored in Assets
// #[derive(Debug)]
// pub struct AnimRef<'a> {
//     pub frames: &'a [u8],
//     pub fps: u8,
//     pub repeat: bool,
//     pub strip: StripID
// }
