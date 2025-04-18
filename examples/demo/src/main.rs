mod backend_cpal;
mod backend_raylib;
mod data;
mod scene_a;
mod scene_b;
mod scene_c;
mod wave_writer;

use backend_raylib::*;
use raylib::{color::Color, texture::Image};
use scene_a::*;
use scene_b::*;
use scene_c::*;
use std::{f32::consts::{PI, TAU}, time::Instant};
use tato::prelude::*;

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

    // Audio setup
    let mut audio_backend = backend_cpal::AudioBackend::new(target_fps);
    audio.sample_rate = audio_backend.sample_rate();
    audio.channels[0].set_volume(15);
    audio.channels[0].set_note(0, 4);
    audio.channels[0].wavetable = WAVE_SAWTOOTH;

    audio_backend.init_audio(&mut audio);
    let note = Note::A3.midi_note();
    let time = Instant::now();
    let mut counter = 0;

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

        let volume = [15, 3, 12];
        audio.channels[0].set_volume(volume[counter % 3]);

        let elapsed = time.elapsed().as_secs_f32();
        let note_offset = ((elapsed * PI).sin()) * 24.0;
        // let mix = ((((elapsed * TAU).sin() + 1.0) / 2.0) * 16.0).min(15.0) as u8;

        // audio.channels[0].set_noise_mix(mix);
        // audio.channels[0].set_volume(volume as u8);

        // audio.channels[0].set_noise_mix(5);
        let stage = counter % 480;
        if stage < 120 {
            // println!("WaveTable");
            audio.channels[0].set_noise_mix(0);
            audio.channels[0].wave_mode = WaveMode::WaveTable;
        } else if stage >= 120 && stage < 240 {
            // println!("Wave Rabndom 1Bit");
            // audio.channels[0].set_noise_mix(0);
            audio.channels[0].wave_mode = WaveMode::Random1Bit;
        } else if stage >= 240 && stage < 360 {
            // println!("Wave Random Sample");
            // audio.channels[0].set_noise_mix(0);
            audio.channels[0].wave_mode = WaveMode::RandomSample;
        } else {
            // println!("Noise");
            // audio.channels[0].set_noise_mix(15);
            audio.channels[0].set_noise_mix(15);
            audio.channels[0].wave_mode = WaveMode::WaveTable;
        }
        audio.channels[0].set_midi_note(note + note_offset);
        audio_backend.process_frame(&mut audio);

        copy_pixels_to_texture(
            &video,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );

        counter += 1;
    }

    audio_backend.wav_file.write_file();
}
