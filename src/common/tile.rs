#![allow(dead_code)]
use super::*;


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileID(pub u16);
impl TileID {
    #[allow(unused)] #[inline]
    pub fn get(self) -> usize { self.0 as usize}
}


#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tile{
    pub index:u8,
    pub flags:u8
}


impl Tile {


    pub fn group(&self) -> u8 { self.flags & 0b0011_1111 }


    pub fn set_group(&mut self, value:u8) {
        if value as usize >= GROUP_COUNT { panic!{"TileFlags: Error, group value range is 0 .. {}. Can't set to {}", GROUP_COUNT-1, value} }
        self.flags &= 0b1100_0000;
        self.flags |= value;
    }
    

    pub fn flipped_h(&self) -> bool { get_bit(self.flags, 0)  }


    pub fn set_flipped_h(&mut self, value:bool) { set_bit(&mut self.flags, value, 0) }


    pub fn flipped_v(&self) -> bool { get_bit(self.flags, 1)  }

    
    pub fn set_flipped_v(&mut self, value:bool) { set_bit(&mut self.flags, value, 1) }

}