
/// The output format of the Pixel Iterator.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct RGBA32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// The Default color is "Debug Pink", not intended to be actually seen!
impl Default for RGBA32{
    fn default() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
            a: 255
        }
    }
}

impl core::fmt::Display for RGBA32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RGBA32(r: {}, g: {}, b: {}, a: {})",
            self.r, self.g, self.b, self.a)
    }
}
