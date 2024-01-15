/// 8 bits per channel representation of an RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub a:u8
}


impl Default for Color {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }
}

impl Color {

    pub fn new(r:u8, g:u8, b:u8, a:u8) -> Color {
        Color{r,g,b,a}
    }

    pub fn black() -> Color { Color { r: 0, g: 0, b: 0, a: 255 } }
    
    pub fn gray_dark() -> Color { Color { r: 48, g: 48, b: 48, a: 255 } }
    
    pub fn gray() -> Color { Color { r: 128, g: 128, b: 128, a: 255 } }
    
    pub fn gray_light() -> Color { Color { r: 192, g: 192, b: 192, a: 255 } }
    
    pub fn white() -> Color { Color { r: 255, g: 255, b: 255, a: 255 } }

    pub fn red() -> Color { Color { r: 255, g: 0, b: 0, a: 255 } }

    pub fn orange() -> Color { Color { r: 255, g: 128, b: 0, a: 255 } }

    pub fn yellow() -> Color { Color { r: 255, g: 255, b: 0, a: 255 } }
    
    pub fn green_light() -> Color { Color { r: 109, g: 255, b: 109, a: 255 } }

    pub fn green_medium() -> Color { Color { r: 36, g: 219, b: 36, a: 255 } }

    pub fn green_dark() -> Color { Color { r: 36, g: 146, b: 36, a: 255 } }

    pub fn blue() -> Color { Color { r: 0, g: 0, b: 255, a: 255 } }
    
}

