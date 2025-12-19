use crate::*;

#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    // Zero is the default mapping, any index above use one of the available mapppings
    pub color_mapping: u8,
    pub group: u8,

}

impl Cell {
    pub const fn new(id:u8, flags:u8, color_mapping:u8, group:u8) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags(flags),
            color_mapping,
            group,
        }
    }

    pub const fn with_id(id:u8) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags::new(false, false, false, false),
            color_mapping: 0,
            group: 0
        }
    }
}
