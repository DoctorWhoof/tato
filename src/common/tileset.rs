/// A subset of an Atlas.
#[derive(Debug)]
pub struct Tileset {
    pub id:u8,
    pub start_index:u16,    // Start Tile index
    pub len:u16,            // Tile count
    pub palette_id:u8,
}

impl Tileset {
    
    pub fn new(id:u8) -> Self {
        Self {
            id,
            start_index: 0,
            len: 0,
            palette_id: 0
        }
    }

}


