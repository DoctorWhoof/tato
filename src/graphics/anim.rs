use crate::*;

#[derive(Debug, Clone)]
pub struct Anim<'a> {
    pub fps: u8,
    pub repeat: bool,
    pub frames: &'a [u8],
    pub strip: &'a [TilemapRef<'a>],
}
