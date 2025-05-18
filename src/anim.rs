use tato_video::Tilemap;

#[derive(Debug, Clone)]
pub struct Anim<const FRAME_COUNT: usize, const FRAME_LEN: usize> {
    pub fps: u8,
    pub cols_per_frame: u8,
    pub frames: [Tilemap<FRAME_LEN>; FRAME_COUNT], // pub frames: [[u8; FRAME_LEN]; FRAME_COUNT],
                                                   // pub flags: [[u8; FRAME_LEN]; LEN],
}

// TODO: Maybe an "AnimPlus" format where the flags are also stored,
// allowing per tile palettes and mirroring?
// As it is, the flags must be passed for the entire animation
// when draing it.

// // An animation contain an array of frames, and a frame is an array of TileIDs
// const ANIM_RUN: Anim<4, 4> = Anim {
//     fps: 10,
//     cols: 2,
//     frames: [
//         [0, 1, 2, 3],
//         [4, 5, 6, 7],
//         [8, 9, 10, 11],
//         [12, 13, 14, 15]
//     ],
// };
