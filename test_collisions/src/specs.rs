use spud::Specs;

#[derive(Clone, Copy)]
pub struct GameSpecs {}
impl Specs for GameSpecs {
    const MAX_COLLIDERS_PER_LAYER:usize = 10;

    const RENDER_WIDTH:usize = 320;
    const RENDER_HEIGHT:usize = 240;

    const TILE_WIDTH:u8 = 16;
    const TILE_HEIGHT:u8 = 16;

    const ATLAS_WIDTH:usize = 128;
    const ATLAS_HEIGHT:usize = 128;

    const MAX_LOADED_TILESETS:usize = 1;
    const MAX_LOADED_FONTS:usize = 1;
    const MAX_LOADED_ANIMS:usize = 1;
    const MAX_LOADED_TILEMAPS:usize = 1;
    const COLORS_PER_PALETTE:usize = 4;
    const ANIMS_PER_TILESET:usize = 1;
    const FONTS_PER_TILESET:usize = 1;
    const TILEMAPS_PER_TILESET:usize = 1;
}

spud::implement_enum_index!(TilesetID);
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TilesetID {
    Default
}

spud::implement_enum_index!(PaletteID);
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum PaletteID {
    Default
}