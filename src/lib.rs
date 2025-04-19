// #![no_std]
mod channel;
mod math;
mod note;
mod rng;

pub mod iter;
pub mod waveform;

pub use channel::*;
pub use note::*;

use core::ops::RangeInclusive;
use rng::Rng;

#[allow(non_camel_case_types)]
pub type u4 = u8;

#[allow(non_camel_case_types)]
pub type i4 = i8;

// "Chip Specs"
// const MIX_COMPRESSION: f32 = 1.6;
const SIZE_U4: u8 = 16;
const SIZE_I4: i8 = 8;

const MAX_I16: f32 = (i16::MAX - 1) as f32; // Technically we don't need to subtract one, but I find it a little safer
const MAX_U4: u8 = SIZE_U4 - 1;
const MAX_I4: i8 = SIZE_I4 - 1;

const MAX_SAMPLE: u8 = 15;
const CHANNEL_COUNT: usize = 4;
const TONE_FREQ_STEPS: u16 = 4096;
// const NOISE_FREQ_STEPS: u16 = 4096;
// const NOISE_PITCH_MULTIPLIER: f32 = 16.0;
const VOLUME_ATTENUATION: f32 = 0.0025;
const VOLUME_EXPONENT: f32 = 2.5;
const FREQ_C4: f32 = 261.63;
// C0 to C10 in "scientific pitch"", roughly the human hearing range
pub const FREQ_RANGE: RangeInclusive<f32> = 16.0..=16384.0;

/// A very simple stereo sample with left and right values.
pub struct Sample<T> {
    pub left: T,
    pub right: T,
}

/// Contains multiple sound channels, and can render and mix them all at once.
#[derive(Debug)]
pub struct AudioChip {
    /// Vector containing sound channels. You can directly manipulate it to add/remove channels.
    pub channels: [Channel; CHANNEL_COUNT],

    // Shared by all channels
    /// Global mix gain, will probably clip audio if more than 1.0 / CHANNEL_COUNT
    pub gain: f32,
    /// The sampling rate at which mixing is performed. Should match your audio playback device,
    /// but can be lower for improved performance. Usually 44100 or 48000.
    pub sample_rate: u32,
    sample_head: usize,
    // Noise
    white_noise: Rng,
    // noise_period: f32,
    // noise_output: f32,
    // noise_period_samples: f32, // Noise period in samples
}

impl Default for AudioChip {
    fn default() -> Self {
        let sample_rate = 48000;
        AudioChip {
            sample_rate,
            channels: core::array::from_fn(|_| Channel::default()),
            // Shared by all channels
            gain: 1.0 / CHANNEL_COUNT as f32,
            sample_head: 0,
            // Noise
            white_noise: Rng::new(16, 0xFACE),
        }
    }
}

impl AudioChip {
    /// Process a single sample, advancing internal timer.
    pub fn process_sample(&mut self) -> Sample<i16> {
        // Generates a new noise sample per step. It's up to the channels to
        // use this sample or not, according to their settings.
        let white_noise = self.white_noise.next_f32();

        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;
        for channel in &mut self.channels {

            let sample = channel.next_sample(self.sample_rate, white_noise);
            // Accumulate channels
            left += sample.left;
            right += sample.right;
        }

        self.sample_head += 1;
        Sample {
            left: ((left * self.gain).clamp(-1.0, 1.0) * MAX_I16) as i16,
            right: ((right * self.gain).clamp(-1.0, 1.0) * MAX_I16) as i16,
        }
    }
}
