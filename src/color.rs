use crate::err;

/// Local Palette index
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct PaletteID(pub u8);

/// Unique identifier for a color in the Global Palette.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorID(pub u8);

/// The output format of the palette. For each ColorID there's a corresponding ColorRGB.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorRGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// The Default color is "Debug Pink", not intended to be actually seen!
impl Default for ColorRGB {
    fn default() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
        }
    }
}

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

pub const BG: ColorID = ColorID(0);
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
pub const PALETTE_DEFAULT: [Color9Bit; 16] = [
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


/// The output format of the palette. For each ColorID there's a corresponding ColorRGB.
#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
pub struct Color9Bit {
    pub data:u16
}


impl Color9Bit {
    pub const fn new(r:u8, g:u8, b:u8) -> Self {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        assert!(g < 8, err!("Exceeded maximum value for Gree channel"));
        assert!(b < 8, err!("Exceeded maximum value for BLue channel"));

        // Pack the 3-bit values into the data field
        // Red in bits 6-8, Green in bits 3-5, Blue in bits 0-2
        let packed_data = ((r as u16) << 6) | ((g as u16) << 3) | (b as u16);
        Self { data: packed_data }
    }
}

impl From<Color9Bit> for ColorRGB {
    fn from(color: Color9Bit) -> Self {
        // Extract the 3-bit color components
        let r = ((color.data >> 6) & 0x7) as u8;
        let g = ((color.data >> 3) & 0x7) as u8;
        let b = (color.data & 0x7) as u8;

        // Scale the 3-bit values (0-7) to 8-bit range (0-255)
        Self {
            r: (r * 36) + (r / 2), // Approximates r * 36.4 without overflow
            g: (g * 36) + (g / 2), // Approximates g * 36.4 without overflow
            b: (b * 36) + (b / 2), // Approximates b * 36.4 without overflow
        }
    }
}
