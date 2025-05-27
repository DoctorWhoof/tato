// use crate::Color14Bit;
use std::collections::HashMap;
use tato_video::*;

// #[derive(Debug, Clone, Copy)]
// pub struct PaletteID(pub u8);

#[derive(Debug, Clone)]
pub(crate) struct PaletteBuilder {
    pub name: String,
    pub colors: Vec<Color12Bit>,
    pub color_hash: HashMap<Color12Bit, u8>,
    id: u8,
}

impl PaletteBuilder {
    pub fn new(name: String, id: u8) -> Self {
        PaletteBuilder {
            name,
            colors: vec![],
            color_hash: HashMap::new(),
            id,
        }
    }

    pub fn push(&mut self, color: Color12Bit) {
        if self.colors.len() == COLORS_PER_PALETTE as usize {
            panic!("Palette error: capacity of {} exceeded.", COLORS_PER_PALETTE)
        }
        self.colors.push(color);
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
