mod backend_cpal;
mod backend_raylib;
mod wave_writer;
pub use wave_writer::*;

use backend_raylib::*;
use raylib::{color::Color, texture::Image};
use std::{f32::consts::PI, time::Instant};
use tato::{prelude::*, Tato};

const W: usize = 240;
const H: usize = 180;
pub const PIXEL_COUNT: usize = W * H * 4;

pub enum SoundType {
    WaveTable,
    WaveRandom,
    WaveRandom1Bit,
    WhiteNoise,
}

// TODO: Remove Videochip since this is an audio demo?
// Can just use raylib to display text on the window

fn main() {
    let mut tato = Tato::new(W as u16, H as u16);

    // Tato Video Setup
    tato.video.bg_color = DARK_BLUE;
    let palette_default = tato.video.push_subpalette([BG_COLOR, LIGHT_BLUE, GRAY, GRAY]);
    let palette_light = tato.video.push_subpalette([BG_COLOR, WHITE, GRAY, GRAY]);
    tato.tiles.new_tile(&TILE_EMPTY);

    // Raylib setup
    let target_fps = 60.0;
    let w = tato.video.width() as i32;
    let h = tato.video.height() as i32;
    let (mut ray, ray_thread) = raylib::init()
        .size(w * 3, h * 3)
        .title("Tato Demo")
        .vsync()
        .resizable()
        .build();
    config_raylib();
    ray.set_target_fps(target_fps as u32);

    // Create texture for rendering
    let mut pixels: [u8; W * H * 4] = core::array::from_fn(|_| 0);
    let mut render_texture = {
        let render_image = Image::gen_image_color(w, h, Color::BLACK);
        ray.load_texture_from_image(&ray_thread, &render_image)
            .unwrap()
    };

    // Load font. TODO: Streamline this.
    for tile in tato::fonts::TILESET_FONT.chunks(64) {
        tato.tiles.new_tile(tile);
    }

    // Pre-draw fixed text (writes to BG Map)
    tato.draw_text(
        "SOUND TEST",
        TextBundle {
            initial_font_tile: 1,
            col: 2,
            row: 2,
            width: 12,
            palette: palette_default,
        },
    );

    tato.draw_text(
        "Currently playing:",
        TextBundle {
            initial_font_tile: 1,
            col: 2,
            row: 6,
            width: 20,
            palette: palette_light,
        },
    );

    // Audio setup
    let mut audio = AudioChip::default();
    let mut audio_backend = backend_cpal::AudioBackend::new(target_fps);
    audio.sample_rate = audio_backend.sample_rate();
    audio.channels[0].set_volume(15);
    audio.channels[0].set_note(Note::C4);
    audio.channels[0].wavetable = WAVE_SQUARE_50;

    audio_backend.init_audio(&mut audio);
    let note = Note::A3.midi_note();
    let time = Instant::now();

    // Main Loop
    while !ray.window_should_close() {
        // "Envelopes"
        let env_len = 2.0;
        let elapsed = time.elapsed().as_secs_f32() % env_len;
        let env = (env_len - elapsed).clamp(0.0, 1.0);
        let note_offset = ((elapsed * PI).sin()) * 24.0;
        audio.channels[0].set_volume((env * 15.0) as u8);
        audio.channels[0].set_note(note + note_offset);

        // Change sound mode depending on time
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

        // Text info
        tato.draw_text(
            &{
                if audio.channels[0].noise_mix() == 0 {
                    format!("Wave Type: {:?}        ", audio.channels[0].wave_mode)
                } else {
                    format!("Wave Type: White Noise        ")
                }
            },
            TextBundle {
                initial_font_tile: 1,
                col: 2,
                row: 8,
                width: 100,
                palette: palette_light,
            },
        );

        tato.draw_text(
            &format!("Volume: {}    ", audio.channels[0].volume()),
            TextBundle {
                initial_font_tile: 1,
                col: 2,
                row: 10,
                width: 100,
                palette: palette_light,
            },
        );

        tato.draw_text(
            &format!("MIDI Note: {:.0}          ", audio.channels[0].midi_note()),
            TextBundle {
                initial_font_tile: 1,
                col: 2,
                row: 12,
                width: 100,
                palette: palette_light,
            },
        );

        // Update backends
        audio_backend.process_frame(&mut audio);
        copy_pixels_to_texture(
            &mut tato,
            &ray_thread,
            &mut ray,
            &mut pixels,
            &mut render_texture,
        );
    }

    audio_backend.write_wav_file();
}
