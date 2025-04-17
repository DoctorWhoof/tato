use core::ops::RangeInclusive;

use crate::{data::WAVE_SQUARE_50, math::*, notes::*, rng::*, *};

const FREQ_C4: f32 = 261.63;

// "Chip Specs"
const TONE_FREQ_STEPS: u16 = 4096;
const NOISE_FREQ_STEPS: u16 = 4096;
const NOISE_PITCH_MULTIPLIER: f32 = 16.0;
const VOLUME_ATTENUATION: f32 = 0.000; // TODO: zeroed for debuggin, unzero

// C0 to C10 in "scientific pitch"", roughly the human hearing range
pub const FREQ_RANGE: RangeInclusive<f32> = 16.0..=16384.0;

#[derive(Debug, Default, Clone, Copy)]
pub enum NoiseMode {
    #[default]
    Noise1Bit,
    WhiteNoise,
    MelodicNoise,
}

/// A single sound channel with configurable properties. Volume is zero by default.
/// There is no "play" or "stop", it simply constantly plays a sound and you manipulate the volume, frequency,
/// pan and noise_mix properties to modify it. If you don't want to waste CPU cycles when it's not playing,
/// simply stop iterating the SoundChip samples instead.
#[derive(Debug)]
pub struct Channel {
    // Main properties
    pub wavetable: [u4; 16], // 0 .. 16 only!
    pub noise_mode: NoiseMode,
    noise_mix: u4,
    volume: u4,
    pan: i4,
    // Timing
    period: f32,
    phase: f32,
    time: f32,
    time_noise: f32,
    // Note cache
    current_midi_note: f32,
    // Noise
    rng: Rng,
    noise_period: f32,
    noise_output: f32,
    // Internal State
    volume_attn: f32,
    wave_output: f32,
    left_mult: f32,
    right_mult: f32,
    last_sample_index: usize,
    last_sample_value: f32,
    last_cycle_index: usize,
}

/// Volume is zero by default! Remember to set each channel volume individually.
impl Default for Channel {
    fn default() -> Self {
        let mut result = Self {
            wavetable: WAVE_SQUARE_50,
            noise_mode: NoiseMode::default(),
            volume: 0,
            pan: 0,
            noise_mix: 0,
            period: 1.0 / FREQ_C4,
            phase: 0.0,
            time: 0.0,
            time_noise: 0.0,
            current_midi_note: get_midi_note(4, 0) as f32,
            rng: Rng::new(16, 0xCAFE),
            noise_period: 0.0,
            noise_output: 0.0,
            volume_attn: 1.0,
            wave_output: 0.0,
            left_mult: 0.5,
            right_mult: 0.5,
            last_sample_index: 0,
            last_sample_value: 0.0,
            last_cycle_index: 0,
        };
        result.set_volume(0);
        result.set_pan(0);
        result.set_noise_mix(0);
        result.set_note(0, 4); // C4
        result
    }
}

impl Channel {
    /// Current frequency. Does not account for pitch envelope.
    pub fn frequency(&self) -> f32 {
        1.0 / self.period
    }

    /// The main volume level.
    pub fn volume(&self) -> u4 {
        self.volume
    }

    /// Current stereo panning. Zero means centered (mono).
    pub fn pan(&self) -> i4 {
        self.pan
    }

    /// Current noise mix. 0 is pure tone, 15 is pure noise.
    pub fn noise_mix(&self) -> u4 {
        self.noise_mix
    }

    pub fn set_volume(&mut self, volume: u4) {
        let volume = volume.min(15);
        self.volume = volume;
        self.calculate_multipliers();
    }

    /// Stereo panning, centered is zero.
    pub fn set_pan(&mut self, pan: i4) {
        let pan = pan.clamp(-7, 7);
        self.pan = pan;
        self.calculate_multipliers();
    }

    /// Switches channel between tone and noise generation, if specs allow noise.
    pub fn set_noise_mix(&mut self, mix: u4) {
        let mix = mix.min(15);
        self.noise_mix = mix;
    }

    /// Adjusts internal pitch values to correspond to octave and note ( where C = 0, C# = 1, etc.).
    pub fn set_note(&mut self, note: impl Into<i32>, octave: impl Into<i32>) {
        let midi_note = get_midi_note(octave, note);
        self.set_midi_note(midi_note as f32);
    }

    /// Same as set_note, but the notes are an f32 value which allows "in-between" notes, or pitch sliding,
    /// and uses MIDI codes instead of octave and note, i.e. C4 is MIDI code 60.
    pub fn set_midi_note(&mut self, note: impl Into<f32>) {
        self.current_midi_note = note.into();
        let frequency = note_to_frequency(self.current_midi_note);
        println!("MIDI: {:.2}", self.current_midi_note);
        println!("freq: {:.2}", frequency);
        self.set_frequency(frequency);
    }

    /// Set the channel's frequency.
    // Private for now, so that the "right" way to set frequency is via note values,
    // and we can easily store those values
    fn set_frequency(&mut self, frequency: f32) {
        // let tone_frequency = quantize_range(frequency, TONE_FREQ_STEPS, FREQ_RANGE);
        let tone_frequency = frequency;

        self.period = 1.0 / tone_frequency;

        // Adjust time to ensure continuous change (instead of abrupt change)
        self.time = self.phase * self.period;

        // SpecsNoise
        let noise_freq = quantize_range(frequency, NOISE_FREQ_STEPS, FREQ_RANGE);
        let noise_period = 1.0 / noise_freq;
        self.noise_period = noise_period / NOISE_PITCH_MULTIPLIER;
    }

    #[inline(always)]
    /// Returns the current sample and peeks the internal timer.
    pub fn next_sample(&mut self, delta_time: f32) -> Sample<f32> {
        // Always apply attenuation, so that values always drift to zero
        self.wave_output *= self.volume_attn;

        // Generate noise level, will be mixed later
        if self.noise_mix > 0 {
            self.noise_output = {
                if self.time_noise >= self.noise_period {
                    self.time_noise = 0.0;
                    let float_noise = self.rng.next_f32();
                    match self.noise_mode {
                        NoiseMode::Noise1Bit => quantize_range(float_noise, 1, -1.0..=1.0),
                        NoiseMode::WhiteNoise => quantize_range(float_noise, u16::MAX, -1.0..=1.0),
                        NoiseMode::MelodicNoise => todo!(),
                    }
                } else {
                    self.noise_output
                }
            };
        }

        // Determine wavetable index
        let len = self.wavetable.len();
        let index = (self.phase * len as f32) as usize;

        // Obtain wavetable sample and set it to output
        if index != self.last_sample_index {
            self.last_sample_index = index;
            let wave = self.wavetable[index];
            let value = wave as f32 / 15.0; // -1.0 to 1.0 range
            // Avoids resetting attenuation if value hasn't changed
            if value != self.last_sample_value {
                // Prevents sampling envelope in the middle of a wave cycle
                let cycle_index = (self.time as f64 / self.period as f64) as usize;
                if cycle_index != self.last_cycle_index {
                    self.last_cycle_index = cycle_index;
                }
                self.wave_output = value;
                self.last_sample_value = value;
            }
        }

        // adjust timers
        self.time += delta_time;
        self.time_noise += delta_time;
        self.phase = (self.time % self.period) / self.period;

        // Mix with noise (currently just overwrites). TODO: optional mix
        // if self.noise_mix > 0 {
        //     let mix = self.noise_mix as f32 / 15.0;
        //     self.wave_output = lerp(self.wave_output, self.noise_output, mix);
        // }

        // Apply main volume
        let output = self.wave_output * (self.volume as f32 / 15.0);

        // Return sample with volume and pan applied
        Sample {
            left: output * self.left_mult,
            right: output * self.right_mult,
        }
    }
}

// Helpers
impl Channel {
    // Must be called after setting volume or pan.
    // Used to pre-calculate as many values as possible instead of doing it per sample, since
    // this function is called much less frequently (by orders of magnitude)
    fn calculate_multipliers(&mut self) {
        debug_assert!(self.pan > -8 && self.pan < 8);
        // Pre calculate this so we don't do it on every sample
        self.volume_attn = 1.0 - VOLUME_ATTENUATION;

        // Pan factor (-1.0 to 1.0)
        let pan_factor = self.pan as f32 / 7.0;

        // Standard equal-power panning
        if pan_factor <= 0.0 {
            self.left_mult = 1.0;
            self.right_mult = 1.0 + pan_factor; // Decreases as pan goes left
        } else {
            self.left_mult = 1.0 - pan_factor; // Decreases as pan goes right
            self.right_mult = 1.0;
        }
    }
}
