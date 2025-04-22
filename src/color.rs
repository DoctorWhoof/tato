/// 8 bits per channel representation of an RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color24 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for Color24 {
    fn default() -> Self {
        Self {
            r: 128,
            g: 64,
            b: 128,
        }
    }
}

impl From<&Color32> for Color24 {
    fn from(value: &Color32) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
        }
    }
}

/// 8 bits per channel representation of an RGBA color.
/// Intended for palette generation preserving transparency information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color32 {
    fn default() -> Self {
        Self {
            r: 128,
            g: 64,
            b: 128,
            a: 255,
        }
    }
}

impl From<&Color24> for Color32 {
    fn from(value: &Color24) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: 255,
        }
    }
}
