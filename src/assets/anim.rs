#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct AnimID(pub u8);

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimEntry {
    pub bank_id: u8,
    pub fps: u8,
    pub columns_per_frame: u8,
    pub rows_per_frame: u8,
    pub data_start: u16,
    pub data_len: u16,
}
