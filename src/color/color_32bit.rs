
/// The output format of the Pixel Iterator.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct ColorRGB32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// The Default color is "Debug Pink", not intended to be actually seen!
impl Default for ColorRGB32{
    fn default() -> Self {
        Self {
            r: 255,
            g: 0,
            b: 255,
            a: 255
        }
    }
}
