// use super::tileset::TilesetID;


/// A contiguous group of tiles in an Atlas (i.e. a text font, or the many tiles of a single object).
#[derive(Clone, Debug, Default)]
pub struct Font {
    pub start: u8,
    pub len: u8,
    pub id: u8,
    pub tileset: u8
}


impl Font {
    
    pub fn last(&self) -> u8 { self.start + self.len - 1 }

}