use crate::Color14Bit;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct PaletteID(pub u8);

#[derive(Debug, Clone)]
pub(crate) struct PaletteBuilder {
    pub name: String,
    pub colors: Vec<Color14Bit>,
    pub color_hash: HashMap<Color14Bit, u8>,
    id: u8,
    palette_head: usize,
}

impl PaletteBuilder {
    pub fn new(name: String, color_count: u8, id: u8) -> Self {
        PaletteBuilder {
            name,
            colors: vec![Color14Bit::default(); color_count as usize],
            color_hash: HashMap::new(),
            palette_head: 0,
            id,
        }
    }

    pub fn push(&mut self, color: Color14Bit) {
        if self.palette_head == self.colors.len() {
            panic!("Palette error: capacity of {} exceeded.", self.colors.len())
        }
        self.colors[self.palette_head] = color;
        self.palette_head += 1;
    }

    pub fn id(&self) -> u8 {
        self.id
    }
}
