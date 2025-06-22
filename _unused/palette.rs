use tato_video::*;

// #[derive(Debug, Clone, Copy, Hash, PartialEq)]
// pub struct PaletteID(pub u8);

// EXPERIMENTAL: This is only visible to the crate, and is used to generate
// a temporary palette before it is loaded into the video chip
#[derive(Debug, Clone, Copy)]
pub(crate) struct Palette {
    // pub bank_id: u8,
    pub colors: [ColorRGBA12; COLORS_PER_PALETTE as usize],
    pub head: u8,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: tato_video::PALETTE_DEFAULT,
            head: 0,
        }
    }
}

impl Palette {

    pub fn reset(&mut self) {
        self.head = 0;
    }

    pub fn push(&mut self, color:ColorRGBA12){
        assert!(self.head < COLORS_PER_PALETTE, err!("Palette capacity exceeded"));
        self.colors[self.head as usize] = color;
        self.head += 1;
    }

}
