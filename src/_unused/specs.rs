
pub struct Specs <
    const PIXEL_COUNT:usize,
    const TILE_COUNT:usize,
    const ANIM_COUNT:usize,
    const FONT_COUNT:usize,
    const TILEMAP_COUNT:usize,
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
//     const ATLAS_ANIM_COUNT:usize,
//     const ATLAS_FONT_COUNT:usize,
//     const ATLAS_TILEMAP_COUNT:usize,
//     // renderer
//     const RENDER_WIDTH:usize,
//     const RENDER_HEIGHT: usize,
//     // World

// >{
//     atlas:Specs< ATLAS_PIXEL_COUNT, ATLAS_TILE_COUNT, ATLAS_ANIM_COUNT, ATLAS_FONT_COUNT, ATLAS_TILEMAP_COUNT >,
//     render: RenderSpecs< RENDER_WIDTH, RENDER_HEIGHT >
// }