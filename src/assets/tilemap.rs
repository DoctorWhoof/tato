#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct MapID(pub u8);

#[derive(Debug, Clone, Copy, Default)]
pub struct TilemapEntry {
    pub columns: u16,
    pub rows: u16,
    pub data_start: u16,
    pub data_len: u16,
    pub bank_id: u8,
}

// pub struct Tilemap<'a> {
//     pub cells: &'a [Cell],
//     pub columns: u16,
//     pub rows: u16,
// }
