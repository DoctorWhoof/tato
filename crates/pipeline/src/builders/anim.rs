use crate::MapBuilder;

#[derive(Debug, Clone, Copy)]
pub struct AnimBuilderID(pub usize);

#[derive(Debug, Clone)]
pub(crate) struct AnimBuilder {
    pub name: String,
    pub frames: Vec<MapBuilder>,
    pub tags: Vec<AnimTag>,
}

#[derive(Debug, Clone)]
pub(crate) struct FrameStep {
    pub index: u8,
    pub duration: u16
}

#[derive(Debug, Clone)]
pub(crate) struct AnimTag{
    pub name: String,
    pub steps: Vec<FrameStep>,
}
