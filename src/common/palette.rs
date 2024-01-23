use super::*;

// #[derive(Debug)]
pub struct Palette<S:Specs>
where [(); S::COLORS_PER_PALETTE]: Sized,
{
    pub(crate) id: Option<u8>,
    pub(crate) colors: [Color; S::COLORS_PER_PALETTE],
    head:usize,
}

impl<S:Specs> Palette<S>
where [(); S::COLORS_PER_PALETTE]: Sized,
{
    pub fn new(from_id:u8) -> Self {
        Palette {
            id: Some(from_id),
            colors: core::array::from_fn(|_| Color::default() ),
            head: 0,
        }
    }


    pub fn id(&self) -> Option<u8> { self.id }

    
    pub fn push(&mut self, color:Color) {
        if self.head == S::COLORS_PER_PALETTE { panic!("Palette error: capacity of {} exceeded.", S::COLORS_PER_PALETTE) }
        self.colors[self.head] = color;
    }
    

    pub fn len(&self) -> usize { self.colors.len() }

}