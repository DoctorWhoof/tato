use crate::Color24;

#[derive(Clone, Copy, Debug, Default)]
pub struct Pixel {
    pub color: Color24,
    pub depth: u8
}