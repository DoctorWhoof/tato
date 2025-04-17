use core::ops::RangeInclusive;

use crate::{data::WAVE_SQUARE_50, math::*, notes::*, rng::*, *};

const FREQ_C4: f32 = 261.63;

// "Chip Specs"
const TONE_FREQ_STEPS: u16 = 4096;
const NOISE_FREQ_STEPS: u16 = 4096;
const NOISE_PITCH_MULTIPLIER: f32 = 16.0;
const VOLUME_ATTENUATION: f32 = 0.002; // TODO: zeroed for debuggin, unzero

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
    pub(crate) sample_rate: f32,
    pub wavetable: [u4; 16], // 0 .. 16 only!
    pub noise_mode: NoiseMode,
    noise_mix: u4,
    volume: u4,
    pan: i4,
    // Timing
    period: f32,
    phase: f32,
    // time_noise: f32,
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
    cycle_index: usize, // Count of complete cycles
    last_cycle_index: usize,
    phase_increment: f32,      // How much to advance phase per sample
    noise_period_samples: f32, // Noise period in samples
}

impl Channel {
    /// Volume is zero by default! Remember to set each channel volume individually.
    pub(crate) fn new(sample_rate: f32) -> Self {
        let mut result = Self {
            sample_rate,
            wavetable: WAVE_SQUARE_50,
            noise_mode: NoiseMode::default(),
            volume: 0,
            pan: 0,
            noise_mix: 0,
            period: 1.0 / FREQ_C4,
            phase: 0.0,
            // time_noise: 0.0,
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
            phase_increment: 0.0,
            noise_period_samples: 0.0,
            cycle_index: 0,
        };
        result.set_volume(0);
        result.set_pan(0);
        result.set_noise_mix(0);
        result.set_note(0, 4); // C4
        result
    }

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
        // println!("MIDI: {:.2}", self.current_midi_note);
        // println!("freq: {:.2}", frequency);
        self.set_frequency(frequency);
    }

    /// Set the channel's frequency.
    // Private for now, so that the "right" way to set frequency is via note values,
    // and we can easily store those values
    fn set_frequency(&mut self, frequency: f32) {
        // Quantize to simulate limited pitch steps
        let tone_frequency = quantize_range(frequency, TONE_FREQ_STEPS, FREQ_RANGE);

        // Calculate how much to advance phase per sample
        self.phase_increment = tone_frequency / self.sample_rate as f32;

        // Keep period for any code that still needs it
        self.period = 1.0 / tone_frequency;

        // SpecsNoise
        let noise_freq = quantize_range(frequency, NOISE_FREQ_STEPS, FREQ_RANGE);
        let noise_period = 1.0 / noise_freq;
        self.noise_period = noise_period / NOISE_PITCH_MULTIPLIER;

        // Calculate noise period in samples
        self.noise_period_samples = (self.sample_rate as f32 * self.noise_period).round();
    }

    #[inline(always)]
    /// Returns the current sample and advances the internal phase by one sample at the configured sample rate
    pub fn next_sample(&mut self) -> Sample<f32> {
        // Always apply attenuation, so that values always drift to zero
        self.wave_output *= self.volume_attn;

        // // Generate noise level, will be mixed later
        // if self.noise_mix > 0 {
        //     self.noise_output = {
        //         // Advance noise phase counter
        //         self.time_noise += 1.0;
        //         if self.time_noise >= self.noise_period_samples {
        //             self.time_noise = 0.0;
        //             let float_noise = self.rng.next_f32();
        //             match self.noise_mode {
        //                 NoiseMode::Noise1Bit => quantize_range(float_noise, 1, -1.0..=1.0),
        //                 NoiseMode::WhiteNoise => quantize_range(float_noise, u16::MAX, -1.0..=1.0),
        //                 NoiseMode::MelodicNoise => todo!(),
        //             }
        //         } else {
        //             self.noise_output
        //         }
        //     };
        // }

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
                if self.cycle_index != self.last_cycle_index {
                    self.last_cycle_index = self.cycle_index;
                    // TODO: Is this doing anything??
                    // UPDATE: Probably used to deal with envelope sampling!
                }
                // Map to (-1.0 .. 1.0) here, ensures proper attenuation over time
                self.wave_output = (value * 2.0) - 1.0;
                self.last_sample_value = value;
            }
        }

        // Advance phase and count cycles properly
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            // Calculate how many complete cycles we've advanced
            let complete_cycles = (self.phase as u32) as usize;
            self.cycle_index += complete_cycles;
            // Keep only the fractional part of phase
            self.phase -= complete_cycles as f32;
        }

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
