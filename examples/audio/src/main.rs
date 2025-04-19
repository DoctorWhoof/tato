mod backend_cpal;
mod backend_raylib;
mod data;
mod debug {
    pub mod wave_writer;
    pub use wave_writer::*;
}

use backend_raylib::*;
use raylib::{color::Color, texture::Image};
use std::{f32::consts::PI, time::Instant};
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

pub enum SoundType {
    WaveTable,
    WaveRandom,
    WaveRandom1Bit,
    WhiteNoise
}

fn main() {
    // Tato setup + initial scene
    let mut video = VideoChip::new(W as u32, H as u32);
    let mut audio = AudioChip::default();
    let mut pad = AnaloguePad::default();
    let mut sound = SoundType::WaveTable;

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
    audio.channels[0].set_note(Note::C4);
    audio.channels[0].wavetable = WAVE_SQUARE_50;

    audio_backend.init_audio(&mut audio);
    let note = Note::A3.midi_note();
    let time = Instant::now();
    let mut frame_count = 0;

    // Main Loop
    while !ray.window_should_close() {
        update_gamepad(&ray, &mut pad);

        let state = BackendState {
            pad,
            time: ray.get_time(),
            elapsed: 1.0 / target_fps,
        };

        // Sound test
        let elapsed = time.elapsed().as_secs_f32() % 2.0;
        let tremolo = [15, 14, 13, 12, 13, 14];
        let env_len = 2.0;
        let env = (env_len - elapsed).clamp(0.0, 1.0);
        let volume = tremolo[frame_count % tremolo.len()] as f32 * env;
        audio.channels[0].set_volume(volume as u8);

        let note_offset = ((elapsed * PI).sin()) * 24.0;
        audio.channels[0].set_note(note + note_offset);

        // let mix = ((((elapsed * TAU).sin() + 1.0) / 2.0) * 16.0).min(15.0) as u8;
        // audio.channels[0].set_noise_mix(mix);

        let stage = time.elapsed().as_secs_f32() % 8.0;
        if stage < 2.0 {
            audio.channels[0].set_noise_mix(0);
            audio.channels[0].wave_mode = WaveMode::WaveTable;
        } else if stage >= 2.0 && stage < 4.0 {
            audio.channels[0].wave_mode = WaveMode::Random1Bit;
        } else if stage >= 4.0 && stage < 6.0 {
            audio.channels[0].wave_mode = WaveMode::RandomSample;
        } else {
            audio.channels[0].set_noise_mix(15);
            audio.channels[0].wave_mode = WaveMode::WaveTable;
        }

        // Update backends
        audio_backend.process_frame(&mut audio);
        copy_pixels_to_texture(
            &video,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );

        frame_count += 1;
    }

    audio_backend.wav_file.write_file();
}
