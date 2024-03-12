
use super::*;
use alloc::{vec, vec::Vec};

#[derive(Debug, Clone)]
pub struct Palette {
    pub(crate) colors: Vec<Color>,
    id: u8,
    head: usize,
}

impl Palette {
    pub fn new(specs:Specs, from_id:u8) -> Self {
        Palette {
            id: from_id,
            colors: vec![Color::default(); specs.colors_per_palette as usize],
            head: 0
        }
    }

    pub fn id(&self) -> u8 { self.id }

    
    pub fn push(&mut self, color:Color) {
        if self.head == self.colors.len() { panic!("Palette error: capacity of {} exceeded.", self.colors.len()) }
        self.colors[self.head] = color;
        self.head += 1;
    }


    pub fn colors(&self) -> &[Color] { self.colors.as_slice() }
    

    pub fn is_empty(&self) -> bool { self.head == 0 }


    pub fn len(&self) -> usize { self.colors.len() }

}