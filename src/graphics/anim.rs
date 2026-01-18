use crate::*;

pub struct Anim<'a, const TILES_PER_FRAME: usize> {
    pub fps: u8,
    pub repeat: bool,
    pub frames: &'a [u8],
    pub strip: &'a [Tilemap<TILES_PER_FRAME>],
}
