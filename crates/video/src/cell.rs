use crate::*;


#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    // pub custom_data: u8, // Unused for now
}
