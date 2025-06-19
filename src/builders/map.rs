use tato_video::*;

#[derive(Debug, Clone)]
pub(crate) struct MapBuilder {
    pub name: String,
    pub cells: Vec<Cell>,
    pub columns: u8,
    pub rows: u8,
}
