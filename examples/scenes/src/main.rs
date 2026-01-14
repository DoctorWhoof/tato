mod scene_a;
mod scene_b;
mod scene_c;

use scene_a::*;
use scene_b::*;
use scene_c::*;
use tato::{
    arena::{Arena, ArenaOps},
    prelude::*,
};

use tato_raylib::*;

// Color mappings. Mappings 2 to 15 map default colors to index 2 (gray)
const COLORMAP_SHADOW: u8 = 0;
const COLORMAP_CYCLE: u8 = 1;

#[derive(Debug, Clone)]
pub struct State {
    pub time: f32,
    pub elapsed: f32,
    pub bg: Tilemap<1600>,
    pub paused: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    tile: TileID,
    flags: TileFlags,
    color_mapping: u8,
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
    None,
    A(SceneA),
    B(SceneB),
    C(SceneC),
}

fn main() -> TatoResult<()> {
    // Tato setup + initial scene
    let mut frame_arena = Arena::<32_768, u32>::new();
    let mut scene = Scene::None;
    let tato = &mut Tato::new(240, 180, 60);
    let dash = &mut Dashboard::new().unwrap();
    let banks = &mut [Bank::new(), Bank::new()];

    let mut state = State {
        time: 0.0,
        elapsed: 0.0,
        bg: Tilemap::<1600>::new(42, 28),
        paused: false,
    };

    // Backend
    let target_fps = 60.0;
    let mut backend = RayBackend::new(&tato);
    backend.print_frame_time = false;
    while !backend.ray.window_should_close() {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);

        // Pausing must happen BEFORE tato.frame_start if we don't
        // want to clear the sprites when paused
        // if backend.get_pressed_key() == Some(Key::Enter) {
        //     tato.paused = !tato.paused
        // }

        tato.frame_start(backend.ray.get_frame_time());
        dash.frame_start(&mut frame_arena, &mut backend);
        state.time = backend.ray.get_time() as f32;
        state.elapsed = 1.0 / target_fps as f32;

        // If scene_change is None, immediately switch to A, otherwise process it.
        let scene_change = match &mut scene {
            Scene::None => Some(SceneChange::A),
            Scene::A(scn) => scn.update(tato, banks, &mut state),
            Scene::B(scn) => scn.update(tato, banks),
            Scene::C(scn) => scn.update(tato, banks),
        };

        // Basic console input
        dash.process_console_line(&mut frame_arena, |_command| Some("Ok".as_bytes()));

        // Update backend
        tato.frame_finish();
        dash.frame_present(&mut frame_arena, banks, &tato, &mut backend);
        backend.frame_present(&mut frame_arena, &tato, banks, &[&state.bg]);

        // Prepare next frame if scene change was requested
        if let Some(choice) = scene_change {
            tato.reset();
            match choice {
                SceneChange::A => scene = Scene::A(SceneA::new(tato, banks, &mut state)?),
                SceneChange::B => scene = Scene::B(SceneB::new(tato, banks, &mut state)?),
                SceneChange::C => scene = Scene::C(SceneC::new(tato, banks, &mut state)?),
            }
        }
    }
    Ok(())
}
