#[derive(Clone)]
pub struct GameSpecs {}
impl spud::Specs for GameSpecs {
    const MAX_COLLIDERS_PER_LAYER:usize = 4;

    const RENDER_WIDTH: usize = 256;
    const RENDER_HEIGHT: usize = 192;

    const TILE_WIDTH: u8 = 8;
    const TILE_HEIGHT: u8 = 8;

    const ATLAS_WIDTH: usize = 128;
    const ATLAS_HEIGHT: usize = 128;

    const MAX_LOADED_TILESETS: usize = 4;
    const MAX_LOADED_FONTS: usize = 1;
    const MAX_LOADED_ANIMS: usize = 4;
    const MAX_LOADED_TILEMAPS: usize = 2;

    const COLORS_PER_PALETTE: usize = 16;
    const ANIMS_PER_TILESET: usize = 4;
    const FONTS_PER_TILESET: usize = 1;
    const TILEMAPS_PER_TILESET: usize = 2;
}
