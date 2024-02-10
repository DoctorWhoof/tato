use crate::*;

const SIZE_OF_ANIM:usize = core::mem::size_of::<Anim>();


/// A collection of frames representing a single action animation (i.e. Idle, Run, etc.). To provide multiple anims to an entity,
/// simply store multiple Anim structs somewhere and assign one of them to the entity.
#[derive(Debug, Clone)]
pub struct  Anim {
    pub id:u8,
    pub group:u8,
    pub fps: u8,
    pub len:u8,  // Actual used length
    pub tileset: u8,
    pub palette: u8,
    pub frames: [Frame; ANIM_MAX_FRAMES],
}

impl Anim {

    pub fn id(&self) -> u8 { self.id }


    pub fn len(&self) -> u8 { self.len }


    pub fn is_empty(&self) -> bool { self.len == 0 }
    

    pub fn frames(&self) -> &[Frame] { &self.frames }


    pub fn serialize(&self) -> [u8; SIZE_OF_ANIM] {
        let mut bytes = ByteArray::<SIZE_OF_ANIM>::new();
        bytes.push(self.id);
        bytes.push(self.group);
        bytes.push(self.fps);
        bytes.push(self.len);
        bytes.push(self.palette);
        bytes.push(self.tileset);
        
        for frame in &self.frames {
            let frame_data = frame.serialize();
            bytes.push_array(&frame_data)
        }
        bytes.validate_and_get_data()
    }

    // Warning: cursor must be already at the position in the array where the Anim block starts
    pub fn deserialize(cursor:&mut Cursor<'_, u8>) -> Self {
        Self {
            id: cursor.next(),
            group: cursor.next(),
            fps: cursor.next(),
            len: cursor.next(),
            palette: cursor.next(),
            tileset: cursor.next(),
            frames: core::array::from_fn(|_| Frame::deserialize(cursor) )
        }
    }
    
    pub fn frame(&self, time:f32) -> &Frame {
        &self.frames[self.current_frame_number(time)]
    }


    pub fn current_frame_number(&self, time:f32) -> usize {
        let interval = 1.0 / self.fps as f32;
        ((time / interval) as usize) % self.len as usize
    }

}





