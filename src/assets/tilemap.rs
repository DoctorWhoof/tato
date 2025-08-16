use tato_arena::Slice;
use tato_video::Cell;

/// A reference to a tilemap associated with a tileset.
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MapID(pub u8);

#[derive(Debug, Default)]
pub(crate) struct TilemapEntry {
    pub cells: Slice<Cell, u16>, // Or store as RawId
    pub columns: u16,
    pub rows: u16,
}
