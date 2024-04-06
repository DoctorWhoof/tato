use super::*;

const SIZE_OF_FONT:usize = core::mem::size_of::<Font>();

/// A contiguous group of tiles representing text characters and numbers.
/// Currently only supports 0 to 9, followed by 'A' to 'Z', upper case only.
#[derive(Clone, Debug)]
pub struct Font {
    pub id: u8,
    pub start_index: u8,
    pub len: u8,
    pub tileset_id: u8
}

pub struct FontInfo {
    pub tileset_id:u8,
    pub font: u8,
    pub depth:u8,
    pub align_right: bool,
}


impl Font {
    
    pub fn last(&self) -> u8 { self.start_index + self.len - 1 }


    pub fn serialize(&self) -> [u8; SIZE_OF_FONT] {
        [
            self.id,
            self.start_index,
            self.len,
            self.tileset_id
        ]
    }

    pub fn deserialize(cursor:&mut Cursor<'_, u8>) -> Self {
        Self {
            id: cursor.advance(),
            start_index: cursor.advance(),
            len: cursor.advance(),
            tileset_id: cursor.advance(),
        }
    }

}