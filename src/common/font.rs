/// A contiguous group of tiles in an Atlas (i.e. a text font, or the many tiles of a single object).
use super::*;

const SIZE_OF_FONT:usize = core::mem::size_of::<Font>();

#[derive(Clone, Debug, Default)]
pub struct Font {
    pub id: u8,
    pub start: u8,
    pub len: u8,
    pub tileset: u8
}


impl Font {
    
    pub fn last(&self) -> u8 { self.start + self.len - 1 }


    pub fn serialize(&self) -> [u8; SIZE_OF_FONT] {
        [
            self.id,
            self.start,
            self.len,
            self.tileset
        ]
    }


    pub fn deserialize(&mut self, cursor:&mut Cursor<'_, u8>) {
        self.id = cursor.next();
        self.start = cursor.next();
        self.len = cursor.next();
        self.tileset = cursor.next();
    }

}