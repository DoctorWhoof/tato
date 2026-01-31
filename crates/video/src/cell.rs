use crate::*;

#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct TileColors(pub u16);

impl TileColors {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        assert!(a < 16, "TileColors: value 'a' exceed maximum of 15");
        assert!(b < 16, "TileColors: value 'b' exceed maximum of 15");
        assert!(c < 16, "TileColors: value 'c' exceed maximum of 15");
        assert!(d < 16, "TileColors: value 'd' exceed maximum of 15");
        let data = (a as u16) << 12 | (b as u16) << 8 | (c as u16) << 4 | d as u16;
        Self(data)
    }

    pub const fn default() -> Self {
        Self::new(0, 1, 2, 3)
    }

    pub fn get(&self, slot: u8) -> usize {
        match slot {
            0 => ((self.0 >> 12) & 15) as usize,
            1 => ((self.0 >> 8) & 15) as usize,
            2 => ((self.0 >> 4) & 15) as usize,
            3 => ((self.0) & 15) as usize,
            _ => panic!("TileColors: Max index is 3, {} was provided", slot),
        }
    }
}

#[derive(Debug, Clone, Default, Copy, PartialEq, Hash)]
pub struct Cell {
    pub id: TileID,
    pub flags: TileFlags,
    pub colors: TileColors, // Zero is the default mapping, any index above use one of the available mapppings
                            // pub color_mapping: u8,
                            // pub group: u8,
}

impl Cell {
    pub const fn new(id: u8, flags: u8, colors: u16) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags(flags),
            colors: TileColors(colors),
        }
    }

    pub const fn with_id(id: u8) -> Self {
        Self {
            id: TileID(id),
            flags: TileFlags::new(false, false, false, false),
            colors: TileColors::default(),
        }
    }
}
