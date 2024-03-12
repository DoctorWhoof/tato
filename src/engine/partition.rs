/// A subset of an Renderer.
#[derive(Debug, Default, Clone)]
pub struct Partition {
    pub id:u8,
    pub previous: Option<u8>,     // The tileset loaded right before this, allows "popping" tilesets
    pub tiles_start_index:u16,    // Start Tile index
    pub tiles_len:u8,            // Tile count
    pub fonts_start_index:u8,
    pub fonts_len:u8,
    pub anims_start_index:u8,
    pub anims_len:u8,
    pub tilemaps_start_index:u8,
    pub tilemaps_len:u8,
    pub debug_palette:u8,
}

