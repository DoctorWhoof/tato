use crate::*;


#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    // pub custom_data: u8, // Unused for now
}
