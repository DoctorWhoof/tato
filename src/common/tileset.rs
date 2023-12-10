use core::sync::atomic::{AtomicU16, Ordering};

static NEXT_ID:AtomicU16 = AtomicU16::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TilesetID{
    pub index:u16,
    pub unique_id:u16
}

impl TilesetID {

    pub fn new(index:u16) -> Self {
        let result = Self {
            index,
            unique_id: NEXT_ID.fetch_add(1, Ordering::AcqRel)
        };
        println!("New TilesetID: {:#?}", result);
        result
    }

    #[allow(unused)] #[inline]
    pub fn get(self) -> usize { self.index as usize}
}


#[derive(Debug, Default)]
pub struct Tileset {
    pub unique_id:u16,
    pub start_index:u16,
    pub len:u16
}


impl Tileset {


}

