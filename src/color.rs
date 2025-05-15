use tato_video::*;

/// Just like Color9Bit, but used as an intermediate format to allow
/// creating correct hashes for tiles with transparent pixels (without this,
/// those pixels are just (0,0,0), which is the same as the color black)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Color14Bit {
    pub data: u16,
    pub alpha: u8,
}

impl Color14Bit {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        assert!(r < 8, err!("Exceeded maximum value for Red channel"));
        assert!(g < 8, err!("Exceeded maximum value for Green channel"));
        assert!(b < 8, err!("Exceeded maximum value for Blue channel"));

        // Pack the 3-bit values into the data field
        // Red in bits 6-8, Green in bits 3-5, Blue in bits 0-2
        let packed_data = ((r as u16) << 6) | ((g as u16) << 3) | (b as u16);
        Self {
            data: packed_data,
            alpha: a,
        }
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
}

impl From<Color14Bit> for ColorRGB32 {
    fn from(color: Color14Bit) -> Self {
        // Extract the 3-bit color components
        let r = ((color.data >> 6) & 0x7) as u8;
        let g = ((color.data >> 3) & 0x7) as u8;
        let b = (color.data & 0x7) as u8;

        // Scale the 3-bit values (0-7) to 8-bit range (0-255)
        Self {
            // Approximate v * 36.4 without overflow
            r: (r * 36) + (r / 2),
            g: (g * 36) + (g / 2),
            b: (b * 36) + (b / 2),
            a: color.alpha,
        }
    }
}

// /// Used when converting from PNG
// #[derive(Debug, Clone, Copy, PartialEq, Hash)]
// pub struct ColorRGB32 {
//     pub r: u8,
//     pub g: u8,
//     pub b: u8,
//     pub a: u8,
// }

// /// The Default color is "Debug Pink", not intended to be actually seen!
// impl Default for ColorRGB32 {
//     fn default() -> Self {
//         Self {
//             r: 255,
//             g: 0,
//             b: 255,
//             a: 255,
//         }
//     }
// }

impl From<ColorRGB32> for Color14Bit {
    fn from(color: ColorRGB32) -> Self {
        // Scale down 8-bit values (0-255) to 3-bit values (0-7)
        // Using (value + 16) / 32 for rounding to nearest value
        let r = ((color.r as u16 + 16) >> 5).min(7);
        let g = ((color.g as u16 + 16) >> 5).min(7);
        let b = ((color.b as u16 + 16) >> 5).min(7);

        // Combine the 3-bit components into a 9-bit value
        Self {
            data: (r << 6) | (g << 3) | b,
            alpha: color.a,
        }
    }
}
