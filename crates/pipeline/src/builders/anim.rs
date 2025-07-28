use crate::MapBuilder;

#[derive(Debug, Clone)]
pub(crate) struct AnimBuilder {
    pub name: String,
    pub frames: Vec<MapBuilder>,
}
