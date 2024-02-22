// use slotmap::SlotMap;

use super::*;
use core::array::from_fn;

// Max 256 tiles
pub struct Tileset<S:Specs>
where
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
{
    pub pixels: [u8; 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)],
    pub fonts: [Option<Font>; S::FONTS_PER_TILESET],
    pub anims: [Option<Anim>; S::ANIMS_PER_TILESET],
    pub tilemaps: [Option<Tilemap>; S::TILEMAPS_PER_TILESET],
    pub debug_palette: u8,
    pub tile_count:u8,
    pub font_count: u8,
    pub anim_count: u8,
    pub tilemap_count: u8,
}


impl<S:Specs> Tileset<S>
where
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
{
    
    /// Returns an empty tileset
    pub fn new() -> Self {
        Self {
            pixels: from_fn(|_| 0),
            fonts: from_fn(|_| None ),
            anims: from_fn(|_| None ),
            tilemaps: from_fn(|_| None ),
            debug_palette: Default::default(),
            tile_count: 0,
            font_count: 0,
            anim_count: 0,
            tilemap_count: 0,
        }
    }

}

impl<S:Specs> Default for Tileset<S>
where
    [(); S::FONTS_PER_TILESET]: Sized,
    [(); S::ANIMS_PER_TILESET]: Sized,
    [(); S::TILEMAPS_PER_TILESET]: Sized,
    [(); 256 * (S::TILE_WIDTH as usize) * (S::TILE_HEIGHT as usize)]: Sized,
{
    fn default() -> Self {
        Self::new()
    }
}