use crate::*;

const SIZE_OF_TILE:usize = core::mem::size_of::<Tile>();

/// Allows recovering the absolute Renderer index from a tile within a tileset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileID(pub u16);
impl TileID {
    #[allow(unused)] #[inline]
    pub fn get(self) -> usize { self.0 as usize}
}

/// The smallest part of a Tilemap, contains a tile index and its flags.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tile{
    pub index:u8,
    pub group:u8,
    pub flags:u8,    //flip_h, flip_v, collider, debug_colliding
}


impl Tile {
    

    pub fn flipped_h(&self) -> bool { get_bit(self.flags, 0)  }


    pub fn set_flipped_h(&mut self, value:bool) { set_bit(&mut self.flags, value, 0) }


    pub fn flipped_v(&self) -> bool { get_bit(self.flags, 1)  }

    
    pub fn set_flipped_v(&mut self, value:bool) { set_bit(&mut self.flags, value, 1) }


    pub fn is_collider(&self) -> bool { get_bit(self.flags, 2)  }


    pub fn set_collider(&mut self, value:bool) { set_bit(&mut self.flags, value, 2) }


    // pub fn is_colliding(&self) -> bool { get_bit(self.flags, 3)  }


    // pub fn set_colliding(&mut self, value:bool) { set_bit(&mut self.flags, value, 3) }


    pub fn serialize(&self) -> [u8; SIZE_OF_TILE] {
        [self.index , self.group, self.flags]
    }

    pub fn deserialize(cursor:&mut Cursor<'_, u8>) -> Self {
        Self {
            index: cursor.advance(),
            group: cursor.advance(),
            flags: cursor.advance(),
        }
    }

}