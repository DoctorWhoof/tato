use tato_arena::{Arena, ArenaOps, Slice};
use tato_video::{Cell, TilemapRef};

/// A reference to a tilemap associated with a tileset.
#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MapID(pub u8);

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct TilemapEntry {
    pub cells: Slice<Cell, u16>, // Or store as RawId
    pub columns: u16,
    pub rows: u16,
}

impl TilemapEntry {
    pub fn to_ref<'a, const LEN: usize>(&self, arena: &'a Arena<LEN, u16>) -> Option<TilemapRef<'a>> {
        let cells = arena.get_slice(self.cells).ok()?;
        Some(TilemapRef { cells, columns: self.columns, rows: self.rows })
    }
}
