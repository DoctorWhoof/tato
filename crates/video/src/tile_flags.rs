use crate::*;

/// A single byte struct that stores a tile's render state such as
/// horizontal flip, vertical flip, rotation and local palette.
#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
pub struct TileFlags(pub u8);

/// A convenient way to pass a palette where TileFlags are required.
/// flip_h and flip_v are left as "false".
impl From<PaletteID> for TileFlags {
    fn from(palette_id: PaletteID) -> TileFlags {
        TileFlags::new(false, false, palette_id)
    }
}

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
    pub const fn with_flip_x(self) -> Self {
        Self(self.0 | 0b_1000_0000)
    }

    /// Consumes the original flag and ensures y is flipped
    pub const fn with_flip_y(self) -> Self {
        Self(self.0 | 0b_0100_0000)
    }

    /// Consumes the original flag and ensures it's rotated 90 degrees
    pub const fn with_rotation(self) -> Self {
        Self(self.0 | 0b_0010_0000)
    }

    /// Consumes the original flag and ensures a BG tiles is rendered in front of sprites.
    pub const fn with_fg(self) -> Self {
        Self(self.0 | 0b_0001_0000)
    }

    /// Consumes the original flag and replaces its palette
    pub const fn with_palette(self, palette: PaletteID) -> Self {
        let data = self.0 & 0b_1111_0000;
        Self(data | palette.0)
    }

    /// Consumes original, sets desired transformations
    pub const fn with_transform(self, flip_x: bool, flip_y: bool, rotation: bool) -> Self {
        let mut flags = self;
        flags.set_flip_x(flip_x);
        flags.set_flip_y(flip_y);
        flags.set_rotation(rotation);
        flags
    }

    pub const fn get_transform_bits(self) -> (bool, bool, bool) {
        (self.is_flipped_x(), self.is_flipped_y(), self.is_rotated())
    }

    pub const fn set_palette(&mut self, palette: PaletteID) {
        debug_assert!(
            palette.0 < SUBPALETTE_COUNT,
            err!("Tile Palette must be in SUBPALETTE_COUNT")
        );
        self.0 &= 0b_1111_0000; // Clear the lower 4 bits (palette bits)
        self.0 |= palette.0 & 0b_0000_1111; // Set the new palette (mask to ensure only lower 4 bits)
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

    /// If true and this is a BG tile, it will be rendered in front of sprites.
    /// This value is ignored when used on sprites.
    pub const fn set_fg(&mut self, state: bool) {
        if state {
            self.0 |= 0b_0001_0000
        } else {
            self.0 &= 0b_1110_1111
        }
    }

    /// Toggles the horizontal flip state
    pub const fn toggle_flip_x(self) -> Self {
        Self(self.0 ^ 0b_1000_0000)
    }

    /// Toggles the vertical flip state
    pub const fn toggle_flip_y(self) -> Self {
        Self(self.0 ^ 0b_0100_0000)
    }

    /// Toggles the rotation state
    pub const fn toggle_rotation(self) -> Self {
        Self(self.0 ^ 0b_0010_0000)
    }

    /// Toggles the foreground state (BG tiles only)
    pub const fn toggle_fg(self) -> Self {
        Self(self.0 ^ 0b_0001_0000)
    }

    pub const fn rotate_up(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= 0b_0000_0000; // set flags
    }

    pub const fn rotate_left(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= 0b_0010_0000; // set flags
    }

    pub const fn rotate_down(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= 0b_0100_0000; // set flags
    }

    pub const fn rotate_right(&mut self) {
        self.0 &= 0b_0001_1111; // clear flags
        self.0 |= 0b_1010_0000; // set flags
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

    /// If true and this is a BG tile, it will be rendered in front of sprites.
    /// This value is ignored when used on sprites.
    // fourth bit
    pub const fn is_fg(&self) -> bool {
        self.0 & 0b_0001_0000 != 0
    }

    /// This flag's palette.
    // Last four bits store the desired palette (0 to 15)
    pub const fn palette(&self) -> PaletteID {
        PaletteID(self.0 & 0b_0000_1111)
    }
}
