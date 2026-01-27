use crate::MapBuilder;

/// Internal builder for animation strips (sequences of tilemaps).
#[derive(Debug, Clone)]
pub(crate) struct StripBuilder {
    pub name: String,
    pub frames: Vec<MapBuilder>,
}

/// Animation definition referencing frames from a strip.
pub struct Anim {
    /// Animation identifier.
    pub name: String,
    /// Playback speed in frames per second.
    pub fps: u8,
    /// If true, animation loops.
    pub repeat: bool,
    /// Frame indices into the strip.
    pub frames: Vec<u8>,
    /// Name of the source animation strip.
    pub strip_name: String,
}
