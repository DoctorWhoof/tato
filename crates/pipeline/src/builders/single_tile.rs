//! Single tile asset definition.

use tato_video::*;

/// Holds a named single-tile asset.
#[derive(Debug, Clone)]
pub(crate) struct SingleTileBuilder {
    /// Tile identifier used in generated code.
    pub name: String,
    /// Cell data for this tile.
    pub cell: Cell,
}
