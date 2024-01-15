use super::*;

// #[derive(Debug)]
pub struct Palette<const CAP:usize> {
    pub id: Option<u8>,
    // pub colors: [Color; CAP],
    pub colors: Pool<Color, CAP>
}

impl<const CAP:usize> Default
for Palette<CAP> {
    fn default() -> Self {
        Self {
            id: None,
            // colors: core::array::from_fn(|_| Color::default())
            colors: Pool::default()
        }
    }
}

impl<const CAP:usize>
Palette<CAP> {

    pub fn new(from_id:u8) -> Self {
        Palette {
            id: Some(from_id),
            ..Default::default()
        }
    }

    
    pub fn push(&mut self, color:Color) {
        self.colors.push(color)
    }
    

    pub fn len(&self) -> usize { self.colors.len() }

}