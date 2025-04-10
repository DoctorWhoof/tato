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
pub const PALETTE_DEFAULT: [ColorRGB; 16] = [
    ColorRGB { r:   0, g:   0, b:   0 }, // BG, 0
    ColorRGB { r:  10, g:  10, b:  10 }, // Black, 1
    ColorRGB { r: 122, g: 122, b: 122 }, // Gray, 2
    ColorRGB { r: 228, g: 228, b: 228 }, // White, 3
    ColorRGB { r: 110, g:  42, b:  20 }, // Dark Red, 4
    ColorRGB { r: 192, g:  64, b:  64 }, // Red, 5
    ColorRGB { r: 212, g: 156, b: 148 }, // Light Red, 6
    ColorRGB { r: 178, g: 126, b:  20 }, // Orange, 7
    ColorRGB { r: 212, g: 202, b:  46 }, // Yellow, 8
    ColorRGB { r:  42, g:  72, b:  48 }, // Dark Green, 9
    ColorRGB { r:  46, g: 154, b:  46 }, // Green, 10
    ColorRGB { r: 140, g: 202, b: 126 }, // Green Light, 11
    ColorRGB { r:  40, g:  40, b: 126 }, // Dark Blue, 12
    ColorRGB { r:  64, g:  72, b: 212 }, // Blue, 13
    ColorRGB { r: 124, g: 176, b: 228 }, // Light Blue, 14
    ColorRGB { r: 188, g:  88, b: 148 }, // Pink, 15
];

/// "Pure" 3 bits per channel palette. Very bright and saturated.
pub const PALETTE_512_MAPPED_TO_16: [ColorRGB; 16] = [
    ColorRGB { r:   0, g:   0, b:   0 }, // BG, 0
    ColorRGB { r:   0, g:   0, b:   0 }, // Black, 1
    ColorRGB { r: 128, g: 128, b: 128 }, // Gray, 2
    ColorRGB { r: 255, g: 255, b: 255 }, // White, 3
    ColorRGB { r:  96, g:  32, b:  32 }, // Dark Red, 4
    ColorRGB { r: 192, g:  32, b:  32 }, // Red, 5
    ColorRGB { r: 255, g: 160, b: 128 }, // Light Red, 6
    ColorRGB { r: 192, g: 128, b:   0 }, // Orange, 7
    ColorRGB { r: 255, g: 255, b:  32 }, // Yellow, 8
    ColorRGB { r:  32, g:  64, b:  32 }, // Dark Green, 9
    ColorRGB { r:  32, g: 160, b:  32 }, // Green, 10
    ColorRGB { r: 128, g: 255, b: 128 }, // Green Light, 11
    ColorRGB { r:  32, g:  32, b: 128 }, // Dark Blue, 12
    ColorRGB { r:  32, g:  64, b: 255 }, // Blue, 13
    ColorRGB { r: 128, g: 255, b: 255 }, // Light Blue, 14
    ColorRGB { r: 192, g:  64, b: 192 }, // Pink, 15
];
