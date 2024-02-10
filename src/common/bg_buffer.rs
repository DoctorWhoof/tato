use super::*;

pub struct BgBuffer {
    pub frame:Frame,
    // Assuming tilemaps aren't humongous, otherwise cols and rows need to be a larger int
    pub source_col:u16,    
    pub source_row:u16,
}
