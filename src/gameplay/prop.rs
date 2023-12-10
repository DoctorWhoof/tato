use crate::engine::EntityID;
use super::*;

#[derive(Debug)]
pub struct Prop {
    pub id: EntityID,
    pub kind: TileKind,
    pub dead_drop: DeadDrop
}

// impl Prop {
//     pub fn with_entity(id:EntityID) -> Self {
//         Prop {
//             id,
//             kind: TileKind::default(),
//             dead_drop: DeadDrop::default()
//         }
//     }
// }