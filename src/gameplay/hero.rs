use crate::*;

pub struct Hero {
    pub id:EntityID,
    pub state: CharState,
    pub health: i8,
    pub inventory: [Item; 10]
}
