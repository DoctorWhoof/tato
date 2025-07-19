use std::{f32::consts::PI, time::Instant};
use tato::{Tato, prelude::*};
use tato_raylib::*;

pub enum SoundType {
    WaveTable,
    WaveRandom,
    WaveRandom1Bit,
    WhiteNoise,
}

fn main() {
    let mut tato = Tato::new(240, 180);
    let mut bg_map = Tilemap::<1024>::new(32, 32);

    // Tato Video Setup
    tato.video.bg_color = RGBA12::DARK_BLUE;
    let palette_default = tato.new_subpalette(0, [BG_COLOR, LIGHT_BLUE, GRAY, GRAY]);
    let palette_light = tato.new_subpalette(0, [BG_COLOR, WHITE, GRAY, GRAY]);

    let _empty = tato.new_tile(0, &DEFAULT_TILES[TILE_EMPTY]); // TODO: Return Option
    let font = tato.push_tileset(0, FONT_TILESET).unwrap();

    // Pre-draw fixed text (writes to BG Map)
    tato.draw_text(
        &mut bg_map,
        "SOUND TEST",
        TextOp {
            id: font,
            col: 2,
            row: 2,
            width: 12,
            palette: palette_default,
        },
    );

    tato.draw_text(
        &mut bg_map,
        "Currently playing:",
        TextOp { id: font, col: 2, row: 6, width: 20, palette: palette_light },
    );

    // Audio setup
    let mut audio = AudioChip::default();
    let mut audio_backend = backend_cpal::AudioBackend::new(60.0);

    audio.sample_rate = audio_backend.sample_rate();
    audio.channels[0].set_volume(15);
    audio.channels[0].set_note(Note::C4);
    audio.channels[0].wavetable = WAVE_SQUARE_50;

    audio_backend.init_audio(&mut audio);
    let note = Note::A3.midi_note();
    let time = Instant::now();

    // Main Loop
    let mut backend = RaylibBackend::new(&tato, 60.0);
    while !backend.ray.window_should_close() {
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
            &mut bg_map,
            &{
                if audio.channels[0].noise_mix() == 0 {
                    format!("Wave Type: {:?}        ", audio.channels[0].wave_mode)
                } else {
                    format!("Wave Type: White Noise        ")
                }
            },
            TextOp { id: font, col: 2, row: 8, width: 100, palette: palette_light },
        );

        tato.draw_text(
            &mut bg_map,
            &format!("Volume: {}    ", audio.channels[0].volume()),
            TextOp {
                id: font,
                col: 2,
                row: 10,
                width: 100,
                palette: palette_light,
            },
        );

        tato.draw_text(
            &mut bg_map,
            &format!("MIDI Note: {:.0}          ", audio.channels[0].midi_note()),
            TextOp {
                id: font,
                col: 2,
                row: 12,
                width: 100,
                palette: palette_light,
            },
        );

        // Update backends
        audio_backend.process_frame(&mut audio);
        backend.render(&mut tato, &[&bg_map]);
    }

    audio_backend.write_wav_file();
}
