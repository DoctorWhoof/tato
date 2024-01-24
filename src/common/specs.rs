pub trait Specs {
    // Renderer
    const RENDER_WIDTH:usize;
    const RENDER_HEIGHT:usize;

    // Atlas
    const TILE_WIDTH:u8;
    const TILE_HEIGHT:u8;

    const ATLAS_WIDTH:usize;
    const ATLAS_HEIGHT:usize; 
    const ATLAS_TILE_COUNT:usize;

    //Assets
    const TILESET_COUNT:usize;
    const ANIM_COUNT:usize;
    const FONT_COUNT:usize;
    const TILEMAP_COUNT:usize;
    const PALETTE_COUNT:usize;
    const COLORS_PER_PALETTE:usize;
} 