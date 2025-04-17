mod backend_raylib;
mod data;
mod scene_a;
mod scene_b;
mod scene_c;
mod wave_writer;

use backend_raylib::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use raylib::{color::Color, texture::Image};
use scene_a::*;
use scene_b::*;
use scene_c::*;
use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender},
};
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

    // CPAL setup
    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let config = device.default_output_config().unwrap();
    let sample_rate = config.sample_rate().0;
    let samples_per_frame = ((sample_rate as f64 / target_fps) * 2.0) as usize + 100;
    println!("Audio sample rate: {}", sample_rate);
    println!("Samples per frame: {}", samples_per_frame);

    // Channel for passing batches of samples
    let (tx, rx): (Sender<Vec<i16>>, Receiver<Vec<i16>>) = mpsc::channel();

    // Audio chip
    audio.channels[0].set_volume(15);
    audio.channels[0].set_note(0, 4);
    audio.sample_rate = config.sample_rate().0;

    // Sample queue for the audio callback
    let mut sample_queue = VecDeque::with_capacity(samples_per_frame * 2); // Extra headroom

    // Start audio stream
    let stream = device
        .build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Get any new batches of samples and add to our queue
                while let Ok(samples) = rx.try_recv() {
                    sample_queue.extend(samples);
                }

                // Fill the output buffer
                for sample_slot in data.iter_mut() {
                    if let Some(sample) = sample_queue.pop_front() {
                        // Convert i16 to float in range [-1.0, 1.0]
                        *sample_slot = sample as f32 / 32768.0;
                    } else {
                        // Queue underrun - fill with silence
                        *sample_slot = 0.0;
                        // You could log underruns during development
                        // println!("Audio buffer underrun");
                    }
                }

                // Optional: Monitor queue size for debugging
                // println!("Queue size: {}", sample_queue.len());
            },
            |err| eprintln!("Audio error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    // Pre-fill the audio buffer before starting the game loop
    // This ensures we have audio ready to play immediately
    let mut startup_samples = Vec::with_capacity(samples_per_frame * 2);
    for _ in 0..(samples_per_frame) {
        let sample = audio.process_sample();
        startup_samples.push(sample.left);
        startup_samples.push(sample.right);
    }
    let _ = tx.send(startup_samples);

    // Set up audio file writing for debugging, check "wave_writer" mod.
    let mut wav_file = wave_writer::WaveWriter::new(audio.sample_rate);

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

        // Generate batch of audio samples for this frame
        let mut frame_samples = Vec::with_capacity(samples_per_frame);
        for _ in 0..samples_per_frame {
            let sample = audio.process_sample();
            frame_samples.push(sample.left);
            frame_samples.push(sample.right);
            wav_file.push(sample.left); // For WAV file debugging
        }

        // Send the entire batch at once
        let _ = tx.send(frame_samples);

        copy_pixels_to_texture(
            &video,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );
    }

    wav_file.write_file();
}
