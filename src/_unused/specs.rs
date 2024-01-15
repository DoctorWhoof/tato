
pub struct AtlasSpecs <
    const PIXEL_COUNT:usize,
    const TILE_COUNT:usize,
    const ANIM_CAP:usize,
    const FONT_CAP:usize,
    const TILEMAP_CAP:usize,
>{}


pub struct RenderSpecs <
    // const WIDTH:usize,
    // const HEIGHT: usize
    const PIXEL_COUNT: usize
>{}


// pub struct WorldSpecs <
//     // Atlas
//     const ATLAS_PIXEL_COUNT:usize,
//     const ATLAS_TILE_COUNT:usize,
//     const ATLAS_ANIM_CAP:usize,
//     const ATLAS_FONT_CAP:usize,
//     const ATLAS_TILEMAP_CAP:usize,
//     // renderer
//     const RENDER_WIDTH:usize,
//     const RENDER_HEIGHT: usize,
//     // World

// >{
//     atlas:AtlasSpecs< ATLAS_PIXEL_COUNT, ATLAS_TILE_COUNT, ATLAS_ANIM_CAP, ATLAS_FONT_CAP, ATLAS_TILEMAP_CAP >,
//     render: RenderSpecs< RENDER_WIDTH, RENDER_HEIGHT >
// }