use crate::*;

/// The smallest part of an animation, contains tiles indices up to ANIM_TILES_PER_FRAME.
#[derive(Debug, Clone)]
pub struct Frame {
    pub cols:u8,
    pub rows:u8,
    pub tiles:[Tile; ANIM_TILES_PER_FRAME]
}


impl Default for Frame {
    fn default() -> Self {
        Self {
            cols:1,
            rows:1,
            tiles: [Tile::default(); ANIM_TILES_PER_FRAME]
        }
    }
}



impl Frame {

    pub fn from_slice(slice:&[Tile], cols:u8, rows:u8) -> Frame {
        let mut tiles = slice.into_iter();
        Frame {
            cols,
            rows,
            tiles: core::array::from_fn(|_| {
                match tiles.next() {
                    Some(tile) => *tile,
                    None => Default::default(),
                }
            }),
        }
    }


    pub fn get_tile(&self, index:u8) -> Tile { self.tiles[index as usize] }

}
