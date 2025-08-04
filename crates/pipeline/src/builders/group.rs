use std::collections::HashMap;
use crate::CanonicalTile;

#[derive(Debug, Default)]
pub(crate) struct GroupBuilder {
    pub hash: HashMap<CanonicalTile, u16>,  // Key: tile, value:group bits
    pub names: Vec<String>                  // Index is group index (0-based), value is group name
}


// #[derive(Debug, Default)]
// pub(crate) struct GroupEntry {
//     pub name:String,
//     pub index:u8,
// }
