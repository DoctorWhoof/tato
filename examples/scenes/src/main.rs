mod scene_a;
mod scene_b;
mod scene_c;

use scene_a::*;
use scene_b::*;
use scene_c::*;
use tato::{Tato, prelude::*};

use tato_raylib::*;

#[derive(Debug, Clone)]
pub struct State {
    pub pad: AnaloguePad,
    pub time: f64,
    pub elapsed: f64,
    pub bg: Tilemap<1600>,
}

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub x: f32,
    pub y: f32,
    tile: TileID,
    flags: TileFlags,
    sub_palette: PaletteID,
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
    let mut scene = Scene::None;
    let mut t = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new();

    let mut state = State {
        pad: t.pad,
        time: 0.0,
        elapsed: 0.0,
        bg: Tilemap::<1600>::new(42, 28),
    };

    // Line scrolling effect, adjusts scroll on every line
    t.video.irq_line = Some(|iter, video, _bg| {
        let line_offset = (iter.y() as f32 + video.scroll_y as f32) / 16.0;
        let phase = ((video.frame_number() as f32 / 30.0) + line_offset).sin();
        iter.scroll_x = (video.scroll_x as f32 - (phase * 8.0)) as i16;
    });

    // Backend
    let target_fps = 60.0;
    let mut back = RaylibBackend::new(&t);
    while !back.ray.window_should_close() {
        t.frame_start(back.ray.get_frame_time());
        back.update_input(&mut t.pad);
        state.time = back.ray.get_time();
        state.elapsed = 1.0 / target_fps as f64;
        state.pad = t.pad;

        // If scene_change is None, immediately switch to A, otherwise process it.
        let scene_change = match &mut scene {
            Scene::None => Some(SceneChange::A),
            Scene::A(scn) => scn.update(&mut t, &mut state),
            Scene::B(scn) => scn.update(&mut t, &mut state),
            Scene::C(scn) => scn.update(&mut t, &mut state),
        };

        // Update backend
        t.frame_finish();
        back.render_canvas(&t, &[&state.bg]);
        back.render_dashboard(&t, &mut dash);
        back.present(&t, Some(&dash));
        dash.clear();

        // Prepare next frame if scene change was requested
        if let Some(choice) = scene_change {
            t.video.reset_all();
            t.reset();
            match choice {
                SceneChange::A => scene = Scene::A(SceneA::new(&mut t, &mut state)?),
                SceneChange::B => scene = Scene::B(SceneB::new(&mut t, &mut state)?),
                SceneChange::C => scene = Scene::C(SceneC::new(&mut t, &mut state)?),
            }
        }
    }
    Ok(())
}
