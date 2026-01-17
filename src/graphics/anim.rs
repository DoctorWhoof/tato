use crate::*;

pub struct Anim<'a, const FRAMES: usize, const TILES_PER_FRAME: usize> {
    pub fps: u8,
    pub rep: bool,
    pub frames: &'a [u8],
    pub strip: &'a [Tilemap<TILES_PER_FRAME>],
}
