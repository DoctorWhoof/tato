// #![no_std]
mod channel;
mod math;
mod notes;
mod rng;

pub mod data;

const MAX_I16: f32 = (i16::MAX - 1) as f32;
const CHANNEL_COUNT: usize = 4;
// const MIX_COMPRESSION: f32 = 1.6;

pub use channel::*;

#[allow(non_camel_case_types)]
pub type u4 = u8;

#[allow(non_camel_case_types)]
pub type i4 = i8;

/// A very simple stereo sample with left and right values.
pub struct Sample<T> {
    pub left: T,
    pub right: T,
}

/// Contains multiple sound channels, and can render and mix them all at once.
#[derive(Debug)]
pub struct AudioChip {
    /// The sampling rate at which mixing is performed. Should match your audio playback device,
    /// but can be lower for improved performance. Usually 44100 or 48000.
    pub sample_rate: u32,
    /// Vector containing sound channels. You can directly manipulate it to add/remove channels.
    pub channels: [Channel; CHANNEL_COUNT],
    /// Global mix gain, will probably clip audio if more than 1.0 / CHANNEL_COUNT
    pub gain: f32,
    sample_head: usize,
}

impl Default for AudioChip {
    fn default() -> Self {
        AudioChip {
            sample_rate: 48000,
            channels: core::array::from_fn(|_| Channel::default()),
            gain: 1.0 / CHANNEL_COUNT as f32,
            sample_head: 0,
        }
    }
}

impl AudioChip {
    /// Process a single sample, advancing internal timer.
    pub fn process_sample(&mut self) -> Sample<i16> {
        // delta will be always tiny, f32 is fine(?)
        let delta_time = 1.0 / self.sample_rate as f32;

        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;
        for channel in &mut self.channels {
            let sample = channel.next_sample(delta_time);
            // Accumulate channels
            left += sample.left;
            right += sample.right;
        }

        self.sample_head += 1;
        let adjusted_left = ((left * 2.0) - 1.0) * self.gain;
        let adjusted_right = ((right * 2.0) - 1.0) * self.gain;
        Sample {
            left: (adjusted_left.clamp(-1.0, 1.0) * MAX_I16) as i16,
            right: (adjusted_right.clamp(-1.0, 1.0) * MAX_I16) as i16,
        }
    }
}

// /// Iterates a specified number of samples. Use [AudioChip::iter()] to obtain this.
// pub struct SoundChipIter<'a> {
//     chip: &'a mut AudioChip,
//     head: usize,
//     sample_count: usize,
// }

// impl<'a> Iterator for SoundChipIter<'a> {
//     type Item = Sample<i16>;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.head < self.sample_count {
//             self.head += 1;
//             return Some(self.chip.process_sample());
//         }
//         None
//     }
// }

// #[inline(always)]
// pub(crate) fn compress_volume(input_vol:f32, max_vol:f32) -> f32 {
//     sinf(input_vol/(max_vol*FRAC_2_PI))
// }
