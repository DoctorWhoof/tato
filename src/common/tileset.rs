
slotmap::new_key_type! { pub struct TilesetID; }


#[derive(Debug, Default)]
pub struct Tileset {
    pub unique_id:TilesetID,
    pub start_index:u16,
    pub len:u16
}

