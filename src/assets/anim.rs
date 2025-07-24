// use tato_video::Tilemap;

// use tato_arena::Pool;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct AnimID(pub u8);

// #[derive(Debug, Clone, Copy, Default)]
// pub(crate) struct AnimEntry {
//     pub bank_id: u8,
//     pub fps: u8,
//     pub columns_per_frame: u8,
//     pub rows_per_frame: u8,
//     pub data_start: u16,
//     pub data_len: u16,
// }

// #[derive(Debug, Clone)]
// pub enum FrameStep {
//     Loop,
//     Hold { frame: u8 },
//     HoldMillisecs { frame: u8, duration: u16 },
//     Custom { code: u16 },
// }

// pub type Anim = Pool<FrameStep>;

// #[derive(Debug, Clone)]
// pub struct Anim<const LEN: usize> {
//     pub steps: [FrameStep; LEN],
// }

pub struct Anim<const LEN:usize> {
    pub fps: u8,
    pub repeat: bool,
    pub frames:[u8; LEN]
}
