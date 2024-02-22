use crate::*;

const SIZE_OF_FRAME:usize = core::mem::size_of::<Frame>();

/// The smallest part of an animation, contains tiles indices up to ANIM_TILES_PER_FRAME.
#[derive(Debug, Clone)]
pub struct Frame {
    pub cols:u8,
    pub rows:u8,
    pub tiles:[Tile; ANIM_TILES_PER_FRAME]
}


// Required to leave certain frames "empty"
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
        let mut tiles = slice.iter();
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


    pub fn serialize(&self) -> [u8; SIZE_OF_FRAME] {
        let mut bytes = ByteArray::<SIZE_OF_FRAME>::new();
        
        bytes.push(self.cols);
        bytes.push(self.rows);
        for tile in self.tiles {
            let tile_data = tile.serialize();
            bytes.push_array(&tile_data)
        }

        bytes.validate_and_get_data()
    }


    pub fn deserialize(cursor:&mut Cursor<'_, u8>) -> Self {
        Frame {
            cols: cursor.advance(),
            rows: cursor.advance(),
            tiles: core::array::from_fn(|_|{
                Tile::deserialize(cursor)
            })
        }
    }


    pub fn get_tile(&self, index:u8) -> Tile { self.tiles[index as usize] }

}
