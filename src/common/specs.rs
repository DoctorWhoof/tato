
pub trait Specs {
    // Renderer
    const RENDER_WIDTH:usize;
    const RENDER_HEIGHT:usize;

    // Atlas
    const TILE_WIDTH:u8;
    const TILE_HEIGHT:u8;

    const ATLAS_WIDTH:usize;
    const ATLAS_HEIGHT:usize; 
    // const ATLAS_TILE_COUNT:usize;

    //Assets
    // const TILESET_COUNT:usize;
    const ANIM_COUNT:usize;
    const FONT_COUNT:usize;
    const TILEMAP_COUNT:usize;
    // const PALETTE_COUNT:usize;
    const COLORS_PER_PALETTE:usize;
} 

// use core::marker::PhantomData;
// use super::*;

// pub struct AssetSpecs <
//     TilesetEnum: IntoPrimitive,
//     PaletteEnum: IntoPrimitive,
//     GroupEnum: IntoPrimitive,
//     AnimEnum: IntoPrimitive,
//     FontEnum: IntoPrimitive,
//     MapEnum: IntoPrimitive
// > {
//     pub tileset:PhantomData<TilesetEnum>,
//     pub palette:PhantomData<PaletteEnum>,
//     pub group:PhantomData<GroupEnum>,
//     pub anim:PhantomData<AnimEnum>,
//     pub font:PhantomData<FontEnum>,
//     pub tilemap:PhantomData<MapEnum>,
// }