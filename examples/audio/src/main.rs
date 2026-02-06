use std::{f32::consts::PI, time::Instant};
use tato::{arena::*, default_assets::*, prelude::*};
use tato_raylib::*;

pub enum SoundType {
    WaveTable,
    WaveRandom,
    WaveRandom1Bit,
    WhiteNoise,
}

fn main() -> TatoResult<()> {
    let mut frame_arena = Arena::<307_200, u32>::new();
    let mut bg_map = Tilemap::<1024>::new(32, 32);
    let mut tato = Tato::new(240, 180, 60);
    let mut dash = Dashboard::new().unwrap();
    let mut banks = [Bank::new()];

    // Tato Video Setup
    tato.video.bg_color = RGBA12::DARK_BLUE;
    banks[0].colors.load_default();
    banks[0].tiles.add(&Tile::default());
    let offset_text = banks[0].append(&BANK_FONT_LONG).unwrap();

    let text_white = &TextOp {
        font: &MAP_FONT_LONG,
        width: None,
        colors: Palette::new(0, 3, 3, 3),
        tile_offset: offset_text,
        character_set: CharacterSet::Long,
    };

    let text_blue = &TextOp {
        font: &MAP_FONT_LONG,
        width: None,
        colors: Palette::new(0, 14, 14, 14),
        tile_offset: offset_text,
        character_set: CharacterSet::Long,
    };

    // Pre-draw fixed text (writes to BG Map)
    draw_text(&mut bg_map, 2, 2, text_blue, "SOUND TEST");
    draw_text(&mut bg_map, 2, 6, text_white, "Currently playing:");

    // Audio setup
    let mut audio = AudioChip::default();
    let mut audio_backend = backend_cpal::AudioBackend::new(&tato);

    audio.sample_rate = audio_backend.sample_rate();
    audio.channels[0].set_volume(15);
    audio.channels[0].set_note(Note::C4);
    audio.channels[0].wavetable = WAVE_SQUARE_50;

    audio_backend.init_audio(&mut audio);
    let note = Note::A3.midi_note();
    let time = Instant::now();

    // Main Loop
    let mut backend = RayBackend::new(&tato);
    while !backend.ray.window_should_close() {
        frame_arena.clear();
        backend.frame_start(&mut frame_arena, &mut tato.pad);

        dash.frame_start(&mut frame_arena, &mut backend);
        tato.frame_start(backend.ray.get_frame_time());

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
        draw_text(&mut bg_map, 2, 8, text_white, &{
            if audio.channels[0].noise_mix() == 0 {
                format!("Wave Type: {:?}        ", audio.channels[0].wave_mode)
            } else {
                format!("Wave Type: White Noise        ")
            }
        });

        draw_text(
            &mut bg_map,
            2,
            10,
            text_white,
            &format!("Volume: {}    ", audio.channels[0].volume()),
        );

        draw_text(
            &mut bg_map,
            2,
            12,
            text_white,
            &format!("MIDI Note: {:.0}          ", audio.channels[0].midi_note()),
        );

        // Update backends
        tato.frame_finish();
        dash.frame_present(&mut frame_arena, &bg_map, &banks, &tato, &mut backend);
        audio_backend.process_frame(&mut audio);
        backend.frame_present(&mut frame_arena, &tato, &banks, &[&bg_map]);
    }

    audio_backend.write_wav_file();
    Ok(())
}
