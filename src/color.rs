mod color_9_bit;
pub use color_9_bit::*;

mod color_24bit;
pub use color_24bit::*;

/// Local Palette index. Each Local palette defines 4 colors out of the 16 "Main" palettes, FG and BG.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct PaletteID(pub u8);

/// Unique identifier for a color in the Main Palettes.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorID(pub u8);

/// Unique identifier for a Local Palette that maps a tile to the global palette.
impl PaletteID {
    pub fn id(self) -> usize {
        self.0 as usize
    }
}

impl ColorID {
    pub fn id(self) -> usize {
        self.0 as usize
    }
}

pub const BG_COLOR: ColorID = ColorID(0);
pub const BLACK: ColorID = ColorID(1);
pub const GRAY: ColorID = ColorID(2);
pub const WHITE: ColorID = ColorID(3);
pub const DARK_RED: ColorID = ColorID(4);
pub const RED: ColorID = ColorID(5);
pub const LIGHT_RED: ColorID = ColorID(6);
pub const ORANGE: ColorID = ColorID(7);
pub const YELLOW: ColorID = ColorID(8);
pub const DARK_GREEN: ColorID = ColorID(9);
pub const GREEN: ColorID = ColorID(10);
pub const GREENLIGHT: ColorID = ColorID(11);
pub const DARK_BLUE: ColorID = ColorID(12);
pub const BLUE: ColorID = ColorID(13);
pub const LIGHT_BLUE: ColorID = ColorID(14);
pub const PINK: ColorID = ColorID(15);

/// Adjusted from the "pure" 512 color palette to be less saturated and less contrasty.
pub(crate) const PALETTE_DEFAULT: [Color9Bit; 16] = [
    Color9Bit::new(0, 0, 0), // BG, 0
    Color9Bit::new(0, 0, 0), // Black, 1
    Color9Bit::new(3, 3, 3), // Gray, 2
    Color9Bit::new(6, 6, 6), // White, 3
    Color9Bit::new(3, 1, 0), // Dark Red, 4
    Color9Bit::new(5, 2, 2), // Red, 5
    Color9Bit::new(6, 4, 4), // Light Red, 6
    Color9Bit::new(5, 3, 0), // Orange, 7
    Color9Bit::new(6, 5, 1), // Yellow, 8
    Color9Bit::new(1, 2, 1), // Dark Green, 9
    Color9Bit::new(1, 4, 1), // Green, 10
    Color9Bit::new(4, 5, 3), // Green Light, 11
    Color9Bit::new(1, 1, 3), // Dark Blue, 12
    Color9Bit::new(2, 2, 6), // Blue, 13
    Color9Bit::new(3, 5, 6), // Light Blue, 14
    Color9Bit::new(5, 2, 4), // Pink, 15
];

// /// "Pure" 3 bits per channel palette. Very bright and saturated.
// pub const PALETTE_DEFAULT: [Color9Bit; 16] = [
//     Color9Bit::new(0, 0, 0), // BG, 0
//     Color9Bit::new(0, 0, 0), // Black, 1
//     Color9Bit::new(4, 4, 4), // Gray, 2
//     Color9Bit::new(7, 7, 7), // White, 3
//     Color9Bit::new(3, 1, 1), // Dark Red, 4
//     Color9Bit::new(5, 1, 1), // Red, 5
//     Color9Bit::new(7, 4, 3), // Light Red, 6
//     Color9Bit::new(5, 3, 0), // Orange, 7
//     Color9Bit::new(7, 7, 1), // Yellow, 8
//     Color9Bit::new(1, 2, 1), // Dark Green, 9
//     Color9Bit::new(1, 4, 1), // Green, 10
//     Color9Bit::new(3, 7, 3), // Green Light, 11
//     Color9Bit::new(1, 1, 3), // Dark Blue, 12
//     Color9Bit::new(1, 2, 7), // Blue, 13
//     Color9Bit::new(3, 7, 7), // Light Blue, 14
//     Color9Bit::new(5, 2, 5), // Pink, 15
// ];
