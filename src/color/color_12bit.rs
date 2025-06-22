use crate::*;

/// Every color in the main palettes (FG and BG palette) is stored as 3 bits-per-channel,
/// allowing a maximum of 512 possible colors packed into 12 bits (excluding alpha).
/// Can be converted to ColorRGBA32 (8 bits per channel) for easy interop with graphics back-ends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ColorRGBA12 {
    pub data: u16,
}

impl ColorRGBA12 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));
        assert!(a < 8, err!("Exceeded maximum value for Alpha channel"));

        // Pack the 3-bit values into the data field
        // Alpha in bits 9-11, Red in bits 6-8, Green in bits 3-5, Blue in bits 0-2
        let packed_data = ((a as u16) << 9) | ((r as u16) << 6) | ((g as u16) << 3) | (b as u16);
        Self { data: packed_data }
    }

    pub fn r(&self) -> u8 {
        (self.data >> 6 & 0b_0111) as u8
    }

    pub fn g(&self) -> u8 {
        (self.data >> 3 & 0b_0111) as u8
    }

    pub fn b(&self) -> u8 {
        (self.data & 0b_0111) as u8
    }

    pub fn a(&self) -> u8 {
        (self.data >> 9 & 0b_0111) as u8
    }

    pub fn set_r(&mut self, r: u8) {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        self.data = (self.data & !(0b_0111 << 6)) | ((r as u16) << 6);
    }

    pub fn set_g(&mut self, g: u8) {
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        self.data = (self.data & !(0b_0111 << 3)) | ((g as u16) << 3);
    }

    pub fn set_b(&mut self, b: u8) {
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));
        self.data = (self.data & !(0b_0111)) | (b as u16);
    }

    pub fn set_a(&mut self, a: u8) {
        assert!(a < 8, err!("Exceeded maximum value for Alpha channel"));
        self.data = (self.data & !(0b_0111 << 9)) | ((a as u16) << 9);
    }
}

impl From<ColorRGBA12> for ColorRGBA32 {
    fn from(color: ColorRGBA12) -> Self {
        // Extract the 3-bit color components
        let r = ((color.data >> 6) & 0x7) as u8;
        let g = ((color.data >> 3) & 0x7) as u8;
        let b = (color.data & 0x7) as u8;
        let a = ((color.data >> 9) & 0x7) as u8;

        // Scale the 3-bit values (0-7) to 8-bit range (0-255)
        Self {
            // Approximate v * 36.4 without overflow
            r: (r * 36) + (r / 2),
            g: (g * 36) + (g / 2),
            b: (b * 36) + (b / 2),
            a: (a * 36) + (a / 2)
        }
    }
}

impl From<ColorRGBA32> for ColorRGBA12 {
    fn from(color: ColorRGBA32) -> Self {
        // Scale down 8-bit values (0-255) to 3-bit values (0-7)
        // Using (value + 16) / 32 for rounding to nearest value
        let r = ((color.r as u16 + 16) >> 5).min(7);
        let g = ((color.g as u16 + 16) >> 5).min(7);
        let b = ((color.b as u16 + 16) >> 5).min(7);
        let a = ((color.a as u16 + 16) >> 5).min(7);

        // Combine the 3-bit components into a 12-bit value
        Self {
            data: (a << 9) | (r << 6) | (g << 3) | b,
        }
    }
}

impl core::fmt::Display for ColorRGBA12 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ColorRGBA12(r: {}, g: {}, b: {}, a: {})",
            self.r(), self.g(), self.b(), self.a())
    }
}
