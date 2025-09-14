use crate::*;

/// A single byte struct that stores a tile's render state such as
/// horizontal flip, vertical flip, rotation and custom data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
pub struct TileFlags(pub u8);

impl TileFlags {
    pub const fn new(flip_h: bool, flip_v: bool, custom_data: u8) -> Self {
        assert!(custom_data < 16, err!("Custom data must be in the 0 to 15 range"));
        let mut data: u8 = 0b_0000_0000;
        // Set tile flip
        if flip_h {
            data |= 0b_1000_0000;
        }
        if flip_v {
            data |= 0b_0100_0000;
        }
        // Set custom data
        let masked_custom_data = custom_data & 0b_0000_1111;
        data |= masked_custom_data;

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

    /// Consumes the original flag and replaces its custom data
    pub const fn with_custom_data(self, custom_data: u8) -> Self {
        let data = self.0 & 0b_1111_0000;
        Self(data | (custom_data & 0b_0000_1111))
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

    pub const fn set_custom_data(&mut self, custom_data: u8) {
        debug_assert!(custom_data < 16, err!("Custom data must be in the 0 to 15 range"));
        self.0 &= 0b_1111_0000; // Clear the lower 4 bits (custom data bits)
        self.0 |= custom_data & 0b_0000_1111; // Set the new custom data (mask to ensure only lower 4 bits)
    }

    /// If true flag will be flipped horizontally
    pub const fn set_flip_x(&mut self, state: bool) {
        if state { self.0 |= 0b_1000_0000 } else { self.0 &= 0b_0111_1111 }
    }

    /// If true flag will be flipped vertically
    pub const fn set_flip_y(&mut self, state: bool) {
        if state { self.0 |= 0b_0100_0000 } else { self.0 &= 0b_1011_1111 }
    }

    /// If true flag will be rotated 90 degrees.
    pub const fn set_rotation(&mut self, state: bool) {
        if state { self.0 |= 0b_0010_0000 } else { self.0 &= 0b_1101_1111 }
    }

    /// If true and this is a BG tile, it will be rendered in front of sprites.
    /// This value is ignored when used on sprites.
    pub const fn set_fg(&mut self, state: bool) {
        if state { self.0 |= 0b_0001_0000 } else { self.0 &= 0b_1110_1111 }
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

    /// This flag's custom data.
    // Last four bits store custom data (0 to 15)
    pub const fn custom_data(&self) -> u8 {
        self.0 & 0b_0000_1111
    }
}
