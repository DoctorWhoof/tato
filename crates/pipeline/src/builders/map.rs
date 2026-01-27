use tato_video::*;

/// Internal representation of a tilemap grid.
#[derive(Debug, Clone)]
pub(crate) struct MapBuilder {
    /// Map identifier.
    pub name: String,
    /// Cell data for each grid position.
    pub cells: Vec<Cell>,
    /// Grid width in tiles.
    pub columns: u8,
    /// Grid height in tiles.
    pub rows: u8,
}
