use tato_video::RGBA12;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ColorEntry {
    /// If true, color already exists in the video chip. If false, it is a new entry.
    pub reused_color: bool,
    /// The index used by the color in the video chip
    pub index: u8,
    /// The color itself
    pub value: RGBA12,
}
