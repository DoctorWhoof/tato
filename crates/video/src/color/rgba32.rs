/// The output format of the Pixel Iterator.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct RGBA32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA32 {
    // Basic color constants
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const GRAY: Self = Self { r: 128, g: 128, b: 128, a: 255 };
    pub const LIGHT_GRAY: Self = Self { r: 192, g: 192, b: 192, a: 255 };
    pub const DARK_GRAY: Self = Self { r: 64, g: 64, b: 64, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Self = Self { r: 255, g: 255, b: 0, a: 255 };
    pub const CYAN: Self = Self { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Self = Self { r: 255, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
}

/// The Default color is "Debug Pink", not intended to be actually seen!
impl Default for RGBA32 {
    fn default() -> Self {
        Self { r: 255, g: 0, b: 255, a: 255 }
    }
}

impl core::fmt::Display for RGBA32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RGBA32(r: {}, g: {}, b: {}, a: {})", self.r, self.g, self.b, self.a)
    }
}
