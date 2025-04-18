use crate::{math::*, waveform::*, *};
use libm::powf;

#[derive(Debug, Default, Clone, Copy)]
pub enum WaveMode {
    #[default]
    WaveTable,
    Random1Bit,
    Random4Bit,
}

/// A single sound channel with configurable properties. Volume is zero by default.
/// There is no "play" or "stop", it simply constantly plays a sound and you manipulate the volume, frequency,
/// pan and noise_mix properties to modify it. If you don't want to waste CPU cycles when it's not playing,
/// simply stop iterating the SoundChip samples instead.
#[derive(Debug)]
pub struct Channel {
    // Main properties
    /// Each sample can have a 0 to 15 value (4 bits). This will be ignore if the
    /// wave_mode is set to anything but WaveTable
    pub wavetable: [u4; 16], // 0 .. 16 only!
    pub wave_mode: WaveMode,
    volume: u4,
    pan: i4,
    noise_mix: u4,
    // Queues wait for next cycle to be applied
    queued_volume: Option<u4>,
    queued_pan: Option<i4>,
    // Misc. Internal State and caches
    frequency: f32,
    phase: f32,
    volume_non_linear: f32,
    volume_attn: f32,
    wave_level: f32, // The "voltage" of the output, drops over time and resets on every new sample
    wave_out: f32,   // The actual output value, combines level, wavetable and volume
    left_mult: f32,
    right_mult: f32,
    last_sample_index: usize,
    last_tone_sample: u4,

    cycle_step: usize, // Zeroed out on every new cycle, used to detect new cycles
}
impl Default for Channel {
    /// Volume is zero by default! Remember to set each channel's volume individually.
    fn default() -> Self {
        let mut result = Self {
            wavetable: WAVE_SAWTOOTH,
            wave_mode: WaveMode::default(),
            volume: 0,
            pan: 0,
            noise_mix: 0,
            // Queues
            queued_volume: None,
            queued_pan: None,
            // Misc. Internal State and caches
            frequency: FREQ_C4,
            phase: 0.0,
            volume_non_linear: 0.0,
            volume_attn: 1.0,
            wave_level: 0.0,
            wave_out: 0.0,
            left_mult: 0.5,
            right_mult: 0.5,
            last_sample_index: 0,
            last_tone_sample: 0,
            cycle_step: 0,
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
        self.frequency
    }

    /// The main volume level.
    pub fn volume(&self) -> u4 {
        self.volume
    }

    /// Current stereo panning. Zero means centered (mono).
    pub fn pan(&self) -> i4 {
        self.pan
    }

    /// Equivalent midi note value for the current frequency
    /// TODO: Needs testing!
    pub fn midi_note(&self) -> f32 {
        frequency_to_note(self.frequency)
    }

    /// Current noise mix. 0 is pure tone, 15 is pure noise.
    pub fn noise_mix(&self) -> u4 {
        self.noise_mix
    }

    pub fn set_volume(&mut self, volume: u4) {
        debug_assert!(
            volume < SIZE_U4,
            "Channel Error: Volume outside allowed range"
        );
        let volume = volume.min(15);
        self.queued_volume = Some(volume);
    }

    /// Stereo panning, centered is zero.
    pub fn set_pan(&mut self, pan: i4) {
        debug_assert!(
            pan < SIZE_I4 && pan > -SIZE_I4,
            "Channel Error: pan outside allowed range"
        );
        let pan = pan.clamp(-7, 7);
        self.queued_pan = Some(pan);
    }

    /// Switches channel between tone and noise generation, if specs allow noise.
    pub fn set_noise_mix(&mut self, mix: u4) {
        debug_assert!(
            mix < SIZE_U4,
            "Channel Error: Noise mix outside allowed range"
        );
        let mix = mix.min(15);
        self.noise_mix = mix;
    }

    /// Adjusts internal pitch values to correspond to octave and note ( where C = 0, C# = 1, etc.).
    pub fn set_note<T>(&mut self, note: T, octave: T)
    where
        T: Into<i32> + Clone,
    {
        debug_assert!(
            {
                let note: i32 = note.clone().into();
                note > -1 && note < 12
            },
            "Channel Error: Note outside allowed range"
        );
        debug_assert!(
            {
                let octave: i32 = octave.clone().into();
                octave > -1 && octave < 11
            },
            "Channel Error: Octave outside allowed range"
        );
        let midi_note = get_midi_note(octave, note);
        self.set_midi_note(midi_note as f32);
    }

    /// Same as set_note, but the notes are an f32 value which allows "in-between" notes, or pitch sliding,
    /// and uses MIDI codes instead of octave and note, i.e. C4 is MIDI code 60.
    pub fn set_midi_note(&mut self, note: impl Into<f32>) {
        let frequency = note_to_frequency(note.into());
        self.set_frequency(frequency);
    }

    /// Set the channel's frequency.
    pub fn set_frequency(&mut self, frequency: f32) {
        // Quantize to simulate limited pitch steps
        self.frequency = quantize_range(frequency, TONE_FREQ_STEPS, FREQ_RANGE);

        // SpecsNoise
        // let noise_freq = quantize_range(frequency, NOISE_FREQ_STEPS, FREQ_RANGE);
        // let noise_period = 1.0 / noise_freq;
        // self.noise_period = noise_period / NOISE_PITCH_MULTIPLIER;

        // Calculate noise period in samples
        // self.noise_period_samples = roundf(self.sample_rate as f32 * self.noise_period);
    }

    #[inline(always)]
    /// Returns the current sample and advances the internal phase by one sample at the configured sample rate
    pub fn next_sample(&mut self, sample_rate: u32, _noise: f32) -> Sample<f32> {
        // Always apply attenuation, so that values always drift to zero
        self.wave_level *= self.volume_attn;

        // Calculate how much to advance phase per sample
        let phase_increment = self.frequency / sample_rate as f32;
        self.phase += phase_increment;
        // Advance phase and count cycles properly
        self.cycle_step += 1;
        if self.phase >= 1.0 {
            // Calculate how many complete cycles we've advanced
            let full_cycles = (self.phase as u32) as usize;
            self.cycle_step = 0;
            // Keep only the fractional part of phase
            self.phase -= full_cycles as f32;
        }

        // Determine wavetable index
        let len = self.wavetable.len();
        let sample_index = (self.phase * len as f32) as usize;

        // // Generate noise level, will be mixed later
        if self.noise_mix > 0 {
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
        }

        // Obtain wavetable sample and set it to output
        if sample_index != self.last_sample_index {
            self.last_sample_index = sample_index;
            let sample = self.wavetable[sample_index].min(MAX_SAMPLE);
            // Avoids resetting attenuation if value hasn't changed
            if sample != self.last_tone_sample {
                let value = sample as f32 / 15.0; // 0.0 to 1.0 range
                // Map to (-1.0 .. 1.0) here, ensures proper attenuation over time
                self.wave_level = (value * 2.0) - 1.0;
                self.last_tone_sample = sample;
            }
            if self.cycle_step == 0 {
                let mut recalc_multipliers = false;
                if let Some(volume) = self.queued_volume {
                    self.volume = volume;
                    self.volume_non_linear = powf(volume as f32 / 15.0, VOLUME_EXPONENT);
                    // println!("{}", self.volume_non_linear);
                    self.queued_volume = None;
                    recalc_multipliers = true;
                }
                if let Some(pan) = self.queued_pan {
                    self.pan = pan;
                    self.queued_pan = None;
                    recalc_multipliers = true;
                }
                if recalc_multipliers {
                    self.calculate_multipliers();
                }
            }
        }

        // Apply main volume
        self.wave_out = if self.volume > 0 {
            // self.wave_level * (self.volume as f32 / 15.0)
            self.wave_level * self.volume_non_linear
        } else {
            self.wave_out * self.volume_attn
        };

        // Return sample with volume and pan applied
        Sample {
            left: self.wave_out * self.left_mult,
            right: self.wave_out * self.right_mult,
        }
    }
}

// Private Helper functions
impl Channel {
    // Must be called after setting volume or pan.
    // Used to pre-calculate as many values as possible instead of doing it per sample, since
    // this function is called much less frequently (by orders of magnitude)
    fn calculate_multipliers(&mut self) {
        debug_assert!(self.pan > -8 && self.pan < 8);
        // Pre calculate this so we don't do it on every sample
        self.volume_attn = 1.0 - VOLUME_ATTENUATION;

        let pan = self.pan as f32 / 7.0; // Maps from -7..=7 to -1.0..=1.0

        self.left_mult = (pan - 1.0) / -2.0;
        self.right_mult = (pan + 1.0) / 2.0;
    }
}
