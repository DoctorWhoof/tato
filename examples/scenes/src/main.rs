mod scene_a;
mod scene_b;
mod scene_c;

use backend_raylib::*;
use raylib::{color::Color, texture::Image};
use scene_a::*;
use scene_b::*;
use scene_c::*;
use tato::{Tato, prelude::*};

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

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
    None,
    A(SceneA),
    B(SceneB),
    C(SceneC),
}

fn main() {
    // Print working directory
    let current_dir = std::env::current_dir().unwrap();
    println!("Current directory: {:?}", current_dir);

    // Tato setup + initial scene
    let mut t = Tato::new(W as u16, H as u16);
    let mut scene = Scene::None;
    // Line scrolling effect, adjusts scroll on every line
    t.video.irq_x_callback = Some(|iter, video, _bg, _tiles| {
        let line_offset = (iter.y() as f32 + video.scroll_y as f32) / 16.0;
        let phase = ((video.frame_count() as f32 / 30.0) + line_offset).sin();
        iter.scroll_x = (video.scroll_x as f32 - (phase * 8.0)) as i16;
    });

    // Raylib setup
    let target_fps = 60.0;
    let w = t.video.width() as i32;
    let h = t.video.height() as i32;
    let (mut ray, ray_thread) = raylib::init()
        .size(w * 3, h * 3)
        .title("Tato Demo")
        .vsync()
        .resizable()
        .build();
    config_raylib();
    ray.set_target_fps(target_fps as u32);

    // Create pixel buffer for rendering
    let mut pixels: [u8; PIXEL_COUNT] = core::array::from_fn(|_| 0);
    let mut render_texture = {
        let render_image = Image::gen_image_color(w, h, Color::BLACK);
        ray.load_texture_from_image(&ray_thread, &render_image)
            .unwrap()
    };

    // Main Loop
    while !ray.window_should_close() {
        update_gamepad(&ray, &mut t.pad);
        let state = BackendState {
            pad: t.pad,
            time: ray.get_time(),
            elapsed: 1.0 / target_fps,
        };

        // If scene is None, immediately switch to A.
        // Otherwise process it to get scene_change.
        let scene_change = match &mut scene {
            Scene::None => Some(SceneChange::A),
            Scene::A(scn) => scn.update(&mut t, state),
            Scene::B(scn) => scn.update(&mut t, state),
            Scene::C(scn) => scn.update(&mut t, state),
        };

        // Update backend
        copy_pixels_to_texture(
            &mut t,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );

        if let Some(choice) = scene_change {
            t.video.reset_all();
            t.tiles.reset();
            match choice {
                SceneChange::A => scene = Scene::A(SceneA::new(&mut t)),
                SceneChange::B => scene = Scene::B(SceneB::new(&mut t)),
                SceneChange::C => scene = Scene::C(SceneC::new(&mut t)),
            }
        }
    }
}
