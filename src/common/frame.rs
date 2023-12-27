use super::tile::*;

pub const MAX_TILES_PER_FRAME:usize = 12;

/// The smallest part of an animation, contains tiles indices up to MAX_TILES_PER_FRAME.
#[derive(Clone)]
pub struct Frame {
    pub(crate) cols:u8,
    pub(crate) rows:u8,
    pub(crate) tiles:[Tile; MAX_TILES_PER_FRAME]
}


impl Default for Frame {
    fn default() -> Self {
        Self {
            cols:1,
            rows:1,
            tiles: [Tile::default(); MAX_TILES_PER_FRAME]
        }
    }
}



impl Frame {

    pub fn get_tile(&self, index:u8) -> Tile { self.tiles[index as usize] }

}
