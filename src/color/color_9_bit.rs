use crate::*;

/// Every color in the main palettes (FG and BG palette) is stored as 3 bits-per-channel,
/// allowing a maximum of 512 possible colors packed into 9 bits.
/// Can be converted to ColorRGB24 (8 bits per channel) for easy interop with graphics back-ends.
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

impl From<Color9Bit> for ColorRGB24 {
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
