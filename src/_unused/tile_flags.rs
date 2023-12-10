use crate::*;


pub const MAX_PANELS:u8 = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TileFlags(pub u8);


impl TileFlags {

    pub fn group(&self) -> u8 { self.0 & 0b0011_1111 }


    pub fn set_group(&mut self, value:u8) {
        if value >= MAX_PANELS { panic!{"TIleFlags: Error, group value is 6 bits(0 ..{}) only. Can't set to {}", MAX_PANELS-1, value} }
        self.0 &= 0b0000_0000;
        self.0 |= value;
    }
    

    pub fn flipped_h(&self) -> bool { get_bit(self.0, 0)  }


    pub fn set_flipped_h(&mut self, value:bool) { set_bit(&mut self.0, value, 0) }


    pub fn flipped_v(&self) -> bool { get_bit(self.0, 1)  }

    
    pub fn set_flipped_v(&mut self, value:bool) { set_bit(&mut self.0, value, 1) }

}