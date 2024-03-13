use core::mem::size_of;

/// 8 bits per channel representation of an RGB color.
/// Intended for framebuffer only, minimal size
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color24 {
    pub r:u8,
    pub g:u8,
    pub b:u8,
}


impl Default for Color24 {
    fn default() -> Self {
        Self { r: 128, g: 64, b: 128 }
    }
}

impl Color24 {

    pub fn new(r:u8, g:u8, b:u8) -> Self {
        Self{r,g,b}
    }

    pub fn black() -> Self { Self { r: 0, g: 0, b: 0 } }
    
    pub fn gray_dark() -> Self { Self { r: 48, g: 48, b: 48 } }
    
    pub fn gray() -> Self { Self { r: 128, g: 128, b: 128 } }
    
    pub fn gray_light() -> Self { Self { r: 192, g: 192, b: 192 } }
    
    pub fn white() -> Self { Self { r: 255, g: 255, b: 255 } }

    pub fn red() -> Self { Self { r: 255, g: 0, b: 0 } }

    pub fn orange() -> Self { Self { r: 255, g: 128, b: 0 } }

    pub fn yellow() -> Self { Self { r: 255, g: 255, b: 0 } }
    
    pub fn green_light() -> Self { Self { r: 109, g: 255, b: 109 } }

    pub fn green_medium() -> Self { Self { r: 36, g: 219, b: 36 } }

    pub fn green_dark() -> Self { Self { r: 36, g: 146, b: 36 } }

    pub fn blue() -> Self { Self { r: 0, g: 0, b: 255 } }
    
}


impl From<&Color32> for Color24 {
    fn from(value: &Color32) -> Self {
        Self{r:value.r, g:value.g, b:value.b}
    }
}



/// 8 bits per channel representation of an RGBA color.
/// Intended for palette generation preserving transparency information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color32 {
    pub r:u8,
    pub g:u8,
    pub b:u8,
    pub a:u8
}


impl Color32 {

    pub fn new(r:u8, g:u8, b:u8, a:u8) -> Self {
        Self{r, g, b, a}
    }

    pub fn serialize(&self) -> [u8; size_of::<Self>()] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn deserialize(data:[u8; size_of::<Self>()]) -> Self  {
        Self{r:data[0], g:data[1], b:data[2], a:data[3]}
    }

}

impl Default for Color32 {
    fn default() -> Self {
        Self { r: 128, g: 64, b: 128, a:255 }
    }
}

impl From<&Color24> for Color32 {
    fn from(value: &Color24) -> Self {
        Self{r:value.r, g:value.g, b:value.b, a:255}
    }
}