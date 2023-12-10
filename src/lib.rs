#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]
// #![no_std]

#[path ="common/_common.rs"] mod common;
#[path ="engine/_engine.rs"] mod engine;

pub use engine::*;
pub use common::*;

// #[cfg(test)]
// mod tests {
//     use crate::World;

//     #[test]
//     fn basic() {
//         let world = World::new();
//     }
// }
