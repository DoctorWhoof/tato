spud::implement_enum_index!(TilesetID);
spud::implement_enum_index!(PaletteID);
spud::implement_enum_index!(GroupID);


#[derive(Clone, Copy)]#[repr(u8)]
pub enum TilesetID {
    Hud,
    Sprites,
    Bg,
}


#[derive(Clone, Copy)]#[repr(u8)]
pub enum PaletteID {
    Main
}


#[derive(Clone, Copy)]#[repr(u8)]
pub enum GroupID {
    None,
    Wall,
    Brick
}