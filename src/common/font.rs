/// A contiguous group of tiles in an Atlas (i.e. a text font, or the many tiles of a single object).
use super::*;

const SIZE_OF_FONT:usize = core::mem::size_of::<Font>();

#[derive(Clone, Debug)]
pub struct Font {
    pub id: u8,
    pub start_index: u8,
    pub len: u8,
    pub tileset_id: u8
}


impl Font {

    
    pub fn new(id:u8) -> Self {
        Self {
            id,
            start_index: 0,
            len: 0,
            tileset_id: 0
        }
    }
    
    pub fn last(&self) -> u8 { self.start_index + self.len - 1 }


    pub fn serialize(&self) -> [u8; SIZE_OF_FONT] {
        [
            self.id,
            self.start_index,
            self.len,
            self.tileset_id
        ]
    }


    pub fn deserialize(&mut self, cursor:&mut Cursor<'_, u8>) {
        self.id = cursor.next();
        self.start_index = cursor.next();
        self.len = cursor.next();
        self.tileset_id = cursor.next();
    }

}