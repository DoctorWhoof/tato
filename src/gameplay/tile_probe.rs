use crate::*;

#[derive(Default, Debug)]
pub struct TileProbe {
    pub tile: Tile,
    pub kind: TileKind,
}


impl TileProbe {

    pub fn check_tile(&mut self, tile_check:Option<Tile>, groups:&[TileKind]) {
        if let Some(tile) = tile_check {
            self.tile = tile;
            self.kind = groups[tile.group() as usize]
        }
    }
    
}