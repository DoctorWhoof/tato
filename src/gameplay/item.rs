// use crate::engine::*;


#[derive(Default, Debug)]
pub enum WeaponKind {
    #[default] Fists,
    // Knife,
    // Gun
}

#[derive(Default, Debug)]
pub enum Item {
    #[default] None,
    // Weapon  { kind:WeaponKind },
    // Key     { quantity:u8 },
    // Flashlight,
    // Umbrella,
    // Camera,
    // Letter
}
