pub const TILESET_HEADER_TEXT:&str = "tileset_1.0";

slotmap::new_key_type! { pub struct TilesetID; }


#[derive(Debug, Default)]
pub struct Tileset {
    pub unique_id:TilesetID,
    pub start_index:u16,
    pub len:u16
}

// // test
// impl Drop for Tileset {
//     fn drop(&mut self) {
//         println!("Dropping Tileset");
//     }
// }

