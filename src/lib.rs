#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]

#![allow(incomplete_features, unstable_features)]
#![feature(generic_const_exprs, variant_count)]

#![no_std]
#[cfg(feature = "std")] extern crate std;
#[cfg(feature = "std")] pub use std::{print, println};

mod test;
#[path ="common/_common.rs"] mod common;
#[path ="engine/_engine.rs"] mod engine;

pub use engine::*;
pub use common::*;

// Consts
pub const GROUP_CAPACITY:usize = 64; // Currently tiles can't represent more than 64 groups (6 bits)

pub const ATLAS_HEADER_TEXT:&str = "atlas_1.0";
pub const TILESET_HEADER_TEXT:&str = "tileset_1.0";
pub const TILEMAP_HEADER_TEXT:&str = "tilemap_1.0" ;

pub const ANIM_MAX_FRAMES:usize = 6;        // TODO: Move to specs
pub const ANIM_TILES_PER_FRAME:usize = 12;  // TODO: Move to specs

pub const TILEMAP_HEADER_LEN:usize = 15;
pub const TILEMAP_LEN:usize = 48 * 48;

pub const COLOR_TRANSPARENCY:Color = Color{r:0,g:255,b:0,a:255};
pub const COLOR_ENTITY_RECT:Color = Color{r:0,g:255,b:255,a:255};
pub const COLOR_COLLIDER:Color = Color{r:255,g:128,b:128,a:255};
pub const COLOR_COLLISION_PROBE:Color = Color{r:255,g:128,b:0,a:255};


