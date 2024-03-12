
#[derive(Clone, Copy, Debug)]
pub struct Specs {
    // FrameBuf
    pub render_width:u16,
    pub render_height:u16,

    // Renderer
    pub atlas_width:u16,
    pub atlas_height:u16, 
    pub tile_width:u8,
    pub tile_height:u8,

    //Assets
    pub colors_per_palette:u8,
} 
