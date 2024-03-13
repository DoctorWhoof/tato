use alloc::{vec, vec::Vec};
use super::*;

/// The main way to organize different tiles and their associated assets: Fonts, Anims and Tilemaps.
pub struct Tileset {
    pub(crate) pixels: Vec<u8>,
    pub(crate) debug_palette: u8,
    fonts: Vec<Font>,
    anims: Vec<Anim>,
    tilemaps: Vec<Tilemap>,
    tile_count:u8,
}


impl Tileset {
    
    /// Returns an empty tileset
    pub fn new(pixel_count:usize, tile_count:u8) -> Self {
        Self {
            pixels: vec![0; pixel_count],
            fonts: vec![],
            anims: vec![],
            tilemaps: vec![],
            debug_palette: Default::default(),
            tile_count,
        }
    }

    pub fn tile_count(&self) -> u8 { self.tile_count }

    pub fn debug_palette(&self) -> u8 { self.debug_palette }

    pub fn anims(&self) -> &Vec<Anim> { &self.anims }

    pub fn fonts(&self) -> &Vec<Font> { &self.fonts }

    pub fn tilemaps(&self) -> &Vec<Tilemap> { &self.tilemaps }

    pub fn anim_count(&self) -> u8 { self.anims.len() as u8 }

    pub fn font_count(&self) -> u8 { self.fonts.len() as u8 }

    pub fn tilemap_count(&self) -> u8 { self.tilemaps.len() as u8 }

    pub fn push_anim(&mut self, anim:Anim) {
        if self.anims.len() < u8::MAX as usize {
            self.anims.push(anim)
        } else {
            panic!("Tileset error: Anim Capacity of 255 exceeded.")
        }
    }

    pub fn push_font(&mut self, font:Font) {
        if self.fonts.len() < u8::MAX as usize {
            self.fonts.push(font)
        } else {
            panic!("Tileset error: Font Capacity of 255 exceeded.")
        }
    }

    pub fn push_tilemap(&mut self, map:Tilemap) {
        if self.tilemaps.len() < u8::MAX as usize {
            self.tilemaps.push(map)
        } else {
            panic!("Tileset error: Tilemap Capacity of 255 exceeded.")
        }
    }

}
