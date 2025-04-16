mod backends;
mod data;
mod scene_a;
mod scene_b;
mod scene_c;

use backends::*;
use scene_a::*;
use scene_b::*;
use scene_c::*;
use tato::{audio::*, prelude::*};

#[derive(Debug, Clone, Copy)]
pub struct BackendState {
    pub pad: AnaloguePad,
    pub time: f64,
    pub elapsed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    tile: TileID,
    flags: TileFlags,
}

// Selects a scene to change into
#[derive(Debug, PartialEq)]
pub enum SceneChange {
    A,
    B,
    C,
}

// Contains the actual scene payload
#[derive(Debug)]
pub enum Scene {
    A(SceneA),
    B(SceneB),
    C(SceneC),
}

fn main() {
    let mut vid = VideoChip::new(240, 180);
    let mut audio = AudioChip::default();
    let mut backend = Backend::new_window(Some(&vid), Some(&audio));
    let mut scene = Scene::A(SceneA::new(&mut vid));

    while !backend.quit_requested() {
        backend.frame_start(&vid);

        let state = BackendState {
            pad: backend.gamepad(),
            time: backend.time(),
            elapsed: backend.elapsed(),
        };

        let scene_change = match &mut scene {
            Scene::A(scn) => scn.update(&mut vid, state),
            Scene::B(scn) => scn.update(&mut vid, state),
            Scene::C(scn) => scn.update(&mut vid, state),
        };

        backend.frame_update(&vid);

        if let Some(choice) = scene_change {
            vid.reset_all();
            match choice {
                SceneChange::A => scene = Scene::A(SceneA::new(&mut vid)),
                SceneChange::B => scene = Scene::B(SceneB::new(&mut vid)),
                SceneChange::C => scene = Scene::C(SceneC::new(&mut vid)),
            }
        }

        backend.frame_finish(&vid);
    }
}
