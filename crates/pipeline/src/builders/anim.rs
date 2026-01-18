use crate::MapBuilder;

#[derive(Debug, Clone)]
pub(crate) struct StripBuilder {
    pub name: String,
    pub frames: Vec<MapBuilder>,
}

pub struct Anim {
    pub name: String,
    pub fps: u8,
    pub repeat: bool,
    pub frames: Vec<u8>,
    pub strip_name: String,
}
