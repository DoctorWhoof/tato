
use super::tileset::TilesetID;

pub const GROUP_COUNT:usize = 16; // Currently can't be higher than 64! 

/// A contiguous group of tiles in an Atlas (i.e. a text font, or the many tiles of a single object).
#[derive(Clone, Copy, Debug)]
pub struct Group {
    pub id: u8,
    pub start:u8,
    pub len:u8,
    pub tileset:TilesetID
}

impl Group {
    
    pub fn last(&self) -> u8 { self.start + self.len - 1 }


    // pub fn next(&self) -> u8 { self.start + self.len }


    pub fn load(bytes:&[u8; 3], tileset:TilesetID) -> Self {
        Group {
            id: bytes[0],
            start: bytes[1],
            len:bytes[2],
            tileset
        }
    }

    // pub fn range(&self) -> RangeInclusive<u8> { self.start ..= (self.start + self.len) }

}