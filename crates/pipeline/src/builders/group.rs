use std::collections::HashMap;
use crate::CanonicalTile;

#[derive(Debug, Default)]
pub (crate) struct GroupBuilder {
    hash: HashMap<CanonicalTile, u16>,  // Key: tile, value:group bits
    names: HashMap<u8, String>          // Key: group index, value: group name (will be used to name a constant later)
}

// #[derive(Debug, Default)]
// pub(crate) struct GroupEntry {
//     pub name:String,
//     pub index:u8,
// }
