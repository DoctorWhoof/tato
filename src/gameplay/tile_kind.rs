#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum TileKind {
    #[default] None,
    Floor,
    Wall,
    Cabinet,
    Door,
    Elevator,
    Stairs,
    // StairsLeft,
    // StairsRight,
    Telephone,
    TrashBin,
    Vase,
}