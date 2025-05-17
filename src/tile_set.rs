
/// A Tileset contains metadata about the location of a group of tiles in a tile bank.
#[derive(Debug, Clone, Default)]
pub struct Tileset {
    pub start:u16,
    pub len: u16,
    pub bank: u8,
    // anims
    // fonts
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
pub struct TilesetID(pub u8);

// impl Tileset {
//     pub fn bank(&self) -> u8 {
//         self.bank
//     }

//     pub fn len(&self) -> usize {
//         self.len as usize
//     }
// }
