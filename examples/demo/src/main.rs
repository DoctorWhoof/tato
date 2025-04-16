mod backend_raylib;
mod data;
mod scene_a;
mod scene_b;
mod scene_c;

use backend_raylib::*;
use raylib::{color::Color, texture::Image};
use scene_a::*;
use scene_b::*;
use scene_c::*;
use tato::{audio::*, prelude::*};

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
    A(SceneA),
    B(SceneB),
    C(SceneC),
}

fn main() {
    // Tato setup + initial scene
    let mut video = VideoChip::new(W as u32, H as u32);
    let mut audio = AudioChip::default();
    let mut pad = AnaloguePad::default();
    let mut scene = Scene::A(SceneA::new(&mut video));

    // Raylib setup
    let target_fps = 60.0;
    let w = video.width() as i32;
    let h = video.height() as i32;
    let (mut ray, ray_thread) = raylib::init()
        .size(w * 3, h * 3)
        .title("Tato Demo")
        .vsync()
        .resizable()
        .build();
    config_raylib();
    ray.set_target_fps(target_fps as u32);

    // Create texture for rendering
    let mut pixels: [u8; PIXEL_COUNT] = core::array::from_fn(|_| 0);
    let mut render_texture = {
        let render_image = Image::gen_image_color(w, h, Color::BLACK);
        ray.load_texture_from_image(&ray_thread, &render_image)
            .unwrap()
    };
    // Audio
    let ray_audio = raylib::core::audio::RaylibAudio::init_audio_device().unwrap();
    let mut audio_stream = ray_audio.new_audio_stream(audio.sample_rate, 16, 2);
    let audio_len = (audio.sample_rate as f64 / target_fps).floor() as usize / 2;
    let mut wave_out = Vec::<i16>::with_capacity(audio_len);
    audio_stream.play();

    // Main Loop
    while !ray.window_should_close() {
        update_gamepad(&ray, &mut pad);

        let state = BackendState {
            pad,
            time: ray.get_time(),
            elapsed: 1.0 / target_fps,
        };

        let scene_change = match &mut scene {
            Scene::A(scn) => scn.update(&mut video, state),
            Scene::B(scn) => scn.update(&mut video, state),
            Scene::C(scn) => scn.update(&mut video, state),
        };

        if let Some(choice) = scene_change {
            video.reset_all();
            match choice {
                SceneChange::A => scene = Scene::A(SceneA::new(&mut video)),
                SceneChange::B => scene = Scene::B(SceneB::new(&mut video)),
                SceneChange::C => scene = Scene::C(SceneC::new(&mut video)),
            }
        }

        copy_pixels_to_texture(
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
            &video,
        );

        // TODO: Audio still broken, write samples to wav file for debugging
        // for _ in 0..audio_len {
        //     let sample = audio.process_sample();
        //     wave_out.push(sample.left);
        //     wave_out.push(sample.right);
        // }
        // audio_stream.update(wave_out.as_slice());
        // wave_out.clear();
    }
}
