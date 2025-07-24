use tato_arena::Pool;
use tato_video::Cell;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MapID(pub u8);

#[derive(Debug, Default)]
pub(crate) struct TilemapEntry {
    pub cells: Pool<Cell, u16>,  // Or store as RawId
    pub columns: u16,
    pub rows: u16,
}
