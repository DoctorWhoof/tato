pub trait Specs: Clone {
    // FrameBuf
    const RENDER_WIDTH:usize;
    const RENDER_HEIGHT:usize;

    // Renderer
    const TILE_WIDTH:u8;
    const TILE_HEIGHT:u8;
    const ATLAS_WIDTH:usize;
    const ATLAS_HEIGHT:usize; 
    const MAX_LOADED_TILESETS:usize;
    const MAX_LOADED_FONTS:usize;
    const MAX_LOADED_ANIMS:usize;
    const MAX_LOADED_TILEMAPS:usize;

    //Assets
    const COLORS_PER_PALETTE:usize;
    const ANIMS_PER_TILESET:usize;
    const FONTS_PER_TILESET:usize;
    const TILEMAPS_PER_TILESET:usize;
} 
