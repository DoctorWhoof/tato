use crate::*;

#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    // pub sub_palette: PaletteID,
    // Zero is the default mapping, any index above use one of the available mapppings
    pub color_mapping: u8,
    pub group: u8,

    // FUTURE USE: In case I abandon subpalettes, colors would be
    // stored as 4 numbers, 4 bits each, like this:
    // pub palette: u16
    // If I want the tile to fit in 32 bits, I'd have to downgrade
    // the group to 4 bits and fit it in flags.
    // 4 groups sounds like... not enough! SO I probably won't do this.
}

impl Cell {
    pub const fn new(id:u8, flags:u8, sub_palette:u8, group:u8) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags(flags),
            // sub_palette: PaletteID(sub_palette),
            color_mapping: 0,
            group,
        }
    }

    pub const fn with_id(self, id:u8) -> Self {
        Self {
            id: TileID(id),
            .. self
        }
    }
}
