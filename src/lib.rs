#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]
// #![no_std]
#![feature(generic_arg_infer)]

#[path ="common/_common.rs"] mod common;
#[path ="engine/_engine.rs"] mod engine;

pub use engine::*;
pub use common::*;

pub const GROUP_CAPACITY:usize = 64; // Currently tiles can't represent more than 64 groups (6 bits)

pub const TILESET_HEADER_TEXT:&str = "tileset_1.0";

pub const ANIM_HEADER_TEXT:&str = "anim_1.0";
pub const ANIM_MAX_FRAMES:usize = 6;
pub const ANIM_HEADER_LEN:usize = ANIM_HEADER_TEXT.len() + 4;
pub const ANIM_TILES_PER_FRAME:usize = 12;

pub const TILEMAP_HEADER_TEXT:&str = "tilemap_1.0" ;
pub const TILEMAP_HEADER_LEN:usize = 15;
pub const TILEMAP_LEN:usize = 48 * 48;

pub const COLOR_TRANSPARENCY:u8 = 255; // The transparent index is hard coded to 255! Allows for black to be 0 and white is 15 in each subpalette.
pub const COLOR_ENTITY_RECT:u8 = 254;
pub const COLOR_COLLIDER:u8 = 253;

// #[cfg(test)]
// mod tests {
//     use crate::World;

//     #[test]
//     fn basic() {
//         let world = World::new();
//     }
// }
