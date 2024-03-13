use super::*;

/// Provides a way for a tilemap to regenerate the bg "behind" a BgSprite (a moving element that writes to the tilemap)
pub struct BgBuffer {
    pub frame:Frame,
    // Assuming tilemaps aren't humongous, otherwise cols and rows need to be a larger int
    pub source_col:u16,
    pub source_row:u16,
}
