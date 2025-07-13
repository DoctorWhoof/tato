mod color_12bit;
pub use color_12bit::*;

mod color_32bit;
pub use color_32bit::*;

/// Local Palette index. Each Local palette defines 4 colors out of the 16 "Main" palettes, FG and BG.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Default)]
pub struct PaletteID(pub u8);

/// Unique identifier for a color in the Main Palettes.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorID(pub u8);

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

impl From<PaletteID> for u8 {
    fn from(value: PaletteID) -> Self {
        value.0
    }
}

impl From<ColorID> for u8 {
    fn from(value: ColorID) -> Self {
        value.0
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

pub const PALETTE_DEFAULT: [RGBA12; 16] = [
    RGBA12::BG,
    RGBA12::BLACK,
    RGBA12::GRAY,
    RGBA12::WHITE,
    RGBA12::DARK_RED,
    RGBA12::RED,
    RGBA12::LIGHT_RED,
    RGBA12::ORANGE,
    RGBA12::YELLOW,
    RGBA12::DARK_GREEN,
    RGBA12::GREEN,
    RGBA12::LIGHT_GREEN,
    RGBA12::DARK_BLUE,
    RGBA12::BLUE,
    RGBA12::LIGHT_BLUE,
    RGBA12::PINK,
];
