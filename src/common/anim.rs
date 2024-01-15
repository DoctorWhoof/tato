use crate::*;

// use slotmap::new_key_type;
// new_key_type! {
//     /// A key to the World slotmap containing animations.
//     pub struct AnimID;
// }



/// A collection of frames representing a single action animation (i.e. Idle, Run, etc.). To provide multiple anims to an entity,
/// simply store multiple Anim structs somewhere and assign one of them to the entity.
#[derive(Debug)]
pub struct  Anim {
    pub group:u8,
    pub fps: u8,
    pub(crate) id:u8,
    pub(crate) len:u8,  // Actual used length
    pub(crate) frames: [Frame; ANIM_MAX_FRAMES],
    pub(crate) tileset: TilesetID
}

impl Default for Anim {
    fn default() -> Self {
        Self {
            id: 0,
            group: Default::default(),
            fps: 10,
            len: 0,
            frames: core::array::from_fn(|_| Frame::default() ),
            tileset: TilesetID::default(),
        }
    }
}


impl Anim {

    pub fn id(&self) -> u8 { self.id }


    pub fn len(&self) -> u8 { self.len }


    pub fn frames(&self) -> &[Frame] { &self.frames }


    pub fn from_frames(frames:[Frame; ANIM_MAX_FRAMES], len:u8, group:u8, fps:u8, id:u8) -> Self {
        Self {
            id,
            group,
            fps,
            len,
            frames,
            tileset: TilesetID::default()
        }
    }

    // pub fn load( data:&[u8], tileset:TilesetID, fps:u8 ) -> Self {
    //     let text_len = ANIM_HEADER_TEXT.len();
    //     if data.len() < ANIM_HEADER_LEN + 1 { panic!("Anim: Invalid .anim file") }
    //     if data[0 .. text_len] != *ANIM_HEADER_TEXT.as_bytes() { panic!("Anim: Invalid .anim file") }

    //     let group_id = data[text_len];
    //     let cols = data[text_len+1];
    //     let rows = data[text_len+2];
    //     let frame_count = data[text_len+3];
    //     let frame_size = cols as usize * rows as usize;

    //     let mut frames:[Frame; ANIM_MAX_FRAMES] = core::array::from_fn(|_| Frame::default() );
    //     (0 .. frame_count as usize).for_each(|frame_index| {
    //         let frame = &mut frames[frame_index];
    //         frame.cols = cols;
    //         frame.rows = rows;
    //         for i in 0 .. frame_size {
    //             let offset = ((frame_index * frame_size) + i) * 2; // Multiplied by 2, index then flags
    //             let index = data[ANIM_HEADER_LEN + offset];
    //             let flags = data[ANIM_HEADER_LEN + offset + 1];
    //             frame.tiles[i] = Tile{index, flags};
    //         }
    //     });

    //     // println!("Anim loaded: {},{},{}", cols, rows, frame_count);
    //     Self {
    //         group_id,
    //         fps,    //TODO: Read from file!
    //         len:frame_count,
    //         frames,
    //         tileset
    //     }
    // }

    
    pub fn frame(&self, time:f32) -> &Frame {
        &self.frames[self.current_frame_number(time)]
    }


    pub fn current_frame_number(&self, time:f32) -> usize {
        let interval = 1.0 / self.fps as f32;
        ((time / interval) as usize) % self.len as usize
    }

}





