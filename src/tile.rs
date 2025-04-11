use super::*;

/// Unique identifier for a tile. Starts at zero when chip is reset.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct TileID(pub u8);

/// Keeps track of a texture tile's dimensions and location in the tile pixel buffer.
#[derive(Debug, Clone, Copy, Default)]
pub struct TileEntry {
    /// Width of the tile.
    pub w: u8,
    /// Height of the tile.
    pub h: u8,
    /// Index to the first cluster of the tile in the tile buffer. Each cluster is 1 byte.
    pub cluster_index: u16,
}

/// A single byte struct that stores a tile's render state such as
/// horizontal flip, vertical flip and local palette.
#[derive(Debug, Clone, Copy, Default)]
pub struct TileFlags(pub u8);

/// A convenient way to pass a palette where TileFlags are required.
/// flip_h and flip_v are left as "false".
impl From<PaletteID> for TileFlags {
    fn from(palette_id: PaletteID) -> TileFlags {
        TileFlags::new(false, false, palette_id)
    }
}

// Currently bits 3 and 4 are unused
impl TileFlags {
    pub const fn new(flip_h: bool, flip_v: bool, palette: PaletteID) -> Self {
        assert!(
            palette.0 < 16,
            err!("Tile Palette must be in the 0 to 15 range")
        );
        let mut data: u8 = 0b_0000_0000;
        // Set tile flip
        if flip_h {
            data |= 0b_1000_0000;
        }
        if flip_v {
            data |= 0b_0100_0000;
        }
        // Set local palette index
        let masked_palette = palette.0 & 0b_0000_1111;
        data |= masked_palette;

        Self(data)
    }

    /// Consumes the original flag and ensures x is flipped
    pub const fn flip_x(self) -> Self {
        Self(self.0 | 0b_1000_0000)
    }

    /// Consumes the original flag and ensures y is flipped
    pub const fn flip_y(self) -> Self {
        Self(self.0 | 0b_0100_0000)
    }

    /// Consumes the original flag and ensures it's rotated 90 degrees
    pub const fn rotate(self) -> Self {
        Self(self.0 | 0b_0010_0000)
    }

    /// Consumes the original flag and ensures it's rendered behind BG tiles.
    pub const fn bg(self) -> Self {
        Self(self.0 | 0b_0001_0000)
    }

    /// Consumes the original flag and replaces its palette
    pub const fn replace_palette(self, palette: PaletteID) -> Self {
        let data = self.0 & 0b_1111_0000;
        Self(data | palette.0)
    }

    /// If true flag will be flipped horizontally
    pub const fn set_flip_x(&mut self, state: bool) {
        if state {
            self.0 |= 0b_1000_0000
        } else {
            self.0 &= 0b_0111_1111
        }
    }

    /// If true flag will be flipped vertically
    pub const fn set_flip_y(&mut self, state: bool) {
        if state {
            self.0 |= 0b_0100_0000
        } else {
            self.0 &= 0b_1011_1111
        }
    }

    /// If true flag will be rotated 90 degrees.
    pub const fn set_rotation(&mut self, state: bool) {
        if state {
            self.0 |= 0b_0010_0000
        } else {
            self.0 &= 0b_1101_1111
        }
    }

    /// If true flag will be rendered behind BG tiles.
    pub const fn set_bg(&mut self, state: bool) {
        if state {
            self.0 |= 0b_0001_0000
        } else {
            self.0 &= 0b_1110_1111
        }
    }

    /// Current horizontal flip state.
    // First bit stores whether the tile is flipped horizontally
    pub const fn is_flipped_x(&self) -> bool {
        self.0 & 0b_1000_0000 != 0
    }

    /// Current vertical flip state.
    // Second bit stores whether the tile is flipped vertically
    pub const fn is_flipped_y(&self) -> bool {
        self.0 & 0b_0100_0000 != 0
    }

    /// Current rotation state.
    // third bit
    pub const fn is_rotated(&self) -> bool {
        self.0 & 0b_0010_0000 != 0
    }

    /// Current rotation state.
    // fourth bit
    pub const fn is_bg(&self) -> bool {
        self.0 & 0b_0001_0000 != 0
    }

    /// This flag's palette.
    // Last four bits store the desired palette (0 to 15)
    pub const fn palette(&self) -> PaletteID {
        PaletteID(self.0 & 0b_0000_1111)
    }
}
