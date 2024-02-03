use super::*;

/// A subset of an Atlas.
#[derive(Debug)]
pub struct Tileset
// pub struct Tileset<S:Specs>
// where
//     [(); S::ANIM_COUNT]: Sized,
//     [(); S::FONT_COUNT]: Sized,
{
    pub id:u8,
    pub start_index:u16,    // Start Tile index
    pub len:u16,            // Tile count
    pub palette_id:u8,
    // pub(crate) fonts: [Font; S::FONT_COUNT],
    // pub(crate) anims: [Anim; S::ANIM_COUNT],
    // pub(crate) tilemaps: [Tilemap; S::TILEMAP_COUNT],
    // pub(crate) rects:[Rect<u8>; (S::ATLAS_WIDTH * S::ATLAS_HEIGHT)/(S::TILE_WIDTH as usize * S::TILE_HEIGHT as usize)],
}

// impl<S:Specs> Tileset<S>
// where
//     [(); S::ANIM_COUNT]: Sized,
//     [(); S::FONT_COUNT]: Sized,
// {
impl Tileset {
    pub fn new(id:u8) -> Self {
        Self {
            id,
            start_index: 0,
            len: 0,
            palette_id: 0,
            // fonts: core::array::from_fn( |i| Font::new( u8::try_from(i).unwrap() )),
            // anims: core::array::from_fn( |i| Anim::new( u8::try_from(i).unwrap() ) ),
        }
    }

}


