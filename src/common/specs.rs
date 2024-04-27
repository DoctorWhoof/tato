
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

    // Assets
    pub colors_per_palette:u8,

    // Max number of assets that can be loaded simultaneously.
    // You can load and unload tilesets (and their associated assets) to avoid hitting this capacity.
    // Pixel capacity is determined by atlas size and tile size.
    pub max_loaded_anims:u8,
    pub max_loaded_fonts:u8,
    pub max_loaded_tilemaps:u8,

} 
