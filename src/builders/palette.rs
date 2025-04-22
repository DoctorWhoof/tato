use std::collections::HashMap;

use tato::video::color::Color9Bit;

#[derive(Debug, Clone, Copy)]
pub struct PaletteID(pub u8);

#[derive(Debug, Clone)]
pub(crate) struct PaletteBuilder {
    pub name: String,
    pub colors: Vec<Color9Bit>,
    pub color_hash: HashMap<Color9Bit, u8>,
    id: u8,
    head: usize,
}

impl PaletteBuilder {

    pub fn new(name:String, color_count:u8, id:u8) -> Self {
        PaletteBuilder {
            name,
            colors: vec![Color9Bit::default(); color_count as usize],
            color_hash: HashMap::new(),
            head: 0,
            id
        }
    }

    pub fn push(&mut self, color:Color9Bit) {
        if self.head == self.colors.len() { panic!("Palette error: capacity of {} exceeded.", self.colors.len()) }
        self.colors[self.head] = color;
        self.head += 1;
    }

    pub fn id(&self) ->u8 {
        self.id
    }


    pub fn colors(&self) -> &[Color9Bit] { self.colors.as_slice() }


    // pub fn is_empty(&self) -> bool { self.head == 0 }


    // pub fn len(&self) -> usize { self.colors.len() }

}
