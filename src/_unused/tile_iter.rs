use crate::*;

struct TileIter {
    visited: [bool; TILEMAP_LEN],
    head:usize
}


impl Iterator for TileIter {
    type Item = Rect<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}