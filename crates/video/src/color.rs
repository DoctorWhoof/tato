mod rgba12;
pub use rgba12::*;

mod rgba32;
pub use rgba32::*;

/// Unique identifier for a color in the Main Palettes.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorID(pub u8);

impl ColorID {
    pub fn id(self) -> usize {
        self.0 as usize
    }
}

impl From<ColorID> for u8 {
    fn from(value: ColorID) -> Self {
        value.0
    }
}

// pub const PALETTE_DEFAULT: [RGBA12; 16] = [
//     RGBA12::TRANSPARENT, // 0
//     RGBA12::BLACK,       // 1
//     RGBA12::GRAY,        // 2
//     RGBA12::WHITE,       // 3
//     RGBA12::DARK_RED,    // 4
//     RGBA12::RED,         // 5
//     RGBA12::LIGHT_RED,   // 6
//     RGBA12::ORANGE,      // 7
//     RGBA12::YELLOW,      // 8
//     RGBA12::DARK_GREEN,  // 9
//     RGBA12::GREEN,       // 10
//     RGBA12::LIGHT_GREEN, // 11
//     RGBA12::DARK_BLUE,   // 12
//     RGBA12::BLUE,        // 13
//     RGBA12::LIGHT_BLUE,  // 14
//     RGBA12::PINK,        // 15
// ];

pub const PALETTE_DEFAULT: [RGBA12; 16] = [
    RGBA12::TRANSPARENT,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
    RGBA12::EMPTY_COLOR,
];
