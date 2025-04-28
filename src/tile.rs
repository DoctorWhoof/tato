use tato_video::{TileFlags, TileID};

/// The smallest part of a BG Map, contains a tile index and its flags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tile {
    pub index: TileID,
    pub flags: TileFlags, //flip_h, flip_v, collider, debug_colliding
    // pub group: u8,
}

// const SIZE_OF_TILE: usize = core::mem::size_of::<Tile>();

// /// Allows recovering the absolute Renderer index from a tile within a tileset.
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub struct TileID(pub u16);
// impl TileID {
//     #[allow(unused)]
//     #[inline]
//     pub fn get(self) -> usize {
//         self.0 as usize
//     }
// }

// impl Tile {
    // pub fn is_collider(&self) -> bool {
    //     get_bit(self.flags, 2)
    // }

    // pub fn set_collider(&mut self, value: bool) {
    //     set_bit(&mut self.flags, value, 2)
    // }

    // pub fn deserialize(cursor: &mut Cursor<'_, u8>) -> Self {
    //     Self {
    //         index: cursor.advance(),
    //         // group: cursor.advance(),
    //         flags: cursor.advance(),
    //     }
    // }
// }
//
