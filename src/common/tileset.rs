// use slotmap::SlotMap;
// use super::*
// slotmap::new_key_type! {
//     /// A key to the World slotmap containing tilesets.
//     pub struct TilesetID;
// }

/// A subset of an Atlas.
#[derive(Debug, Default)]
pub struct Tileset {
    pub unique_id:u8,
    pub start_index:u16,    // Start Tile index
    pub len:u16,            // Tile count
    pub palette_id:u8,
    // pub groups:SlotMap<GroupID, Group>,
    // pub anims:SlotMap<AnimID, Anim>
}

// // test
// impl Drop for Tileset {
//     fn drop(&mut self) {
//         println!("Dropping Tileset");
//     }
// }

