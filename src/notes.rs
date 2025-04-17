use crate::math::*;

/// Returns the MIDI note value given an octave (zero to 10) and a note (zero to 11).
pub fn get_midi_note(octave: impl Into<i32>, note: impl Into<i32>) -> i32 {
    // Handle negative values and values beyond range
    let octave = wrap(octave.into(), 10);
    let note = wrap(note.into(), 12);
    // MIDI note number, where C4 is 60
    ((octave + 1) * 12) + note
}

// TODO: Needs testing!
/// Converts a MIDI note number to frequency in Hz
#[inline(always)]
pub fn note_to_frequency(note: f32) -> f32 {
    // A4 (MIDI note 69) = 440 Hz
    const A4_FREQ: f32 = 440.0;

    // Each octave is 12 semitones and doubles the frequency
    let semitones_from_a4 = note - 69.0;

    // Split into octaves and semitones within octave
    let octaves = trunc(semitones_from_a4 / 12.0);
    let semitones = semitones_from_a4 - (octaves * 12.0);

    // Handle negative semitones by normalizing to 0-11.999... range
    let normalized_semitones = if semitones < 0.0 {
        semitones + 12.0
    } else {
        semitones
    };

    // Pre-computed semitone ratios for exactly one octave
    const SEMITONE_RATIOS: [f32; 13] = [
        1.0,         // 0 semitones from reference
        1.059463,    // 1 semitone
        1.122462,    // 2 semitones
        1.189207,    // 3 semitones
        1.259921,    // 4 semitones
        1.334840,    // 5 semitones
        1.414214,    // 6 semitones
        1.498307,    // 7 semitones
        1.587401,    // 8 semitones
        1.681793,    // 9 semitones
        1.781797,    // 10 semitones
        1.887749,    // 11 semitones
        2.0,         // 12 semitones (one octave)
    ];

    // Get the appropriate semitone ratio using linear interpolation
    let semitone_index = floor(normalized_semitones) as usize;
    let fraction = normalized_semitones - floor(normalized_semitones);

    let ratio = if fraction > 0.0 && semitone_index < 12 {
        let ratio1 = SEMITONE_RATIOS[semitone_index];
        let ratio2 = SEMITONE_RATIOS[semitone_index + 1];
        ratio1 + fraction * (ratio2 - ratio1)
    } else {
        SEMITONE_RATIOS[semitone_index]
    };

    // Calculate octave adjustment, accounting for negative semitones
    let adjusted_octaves = if semitones < 0.0 {
        octaves - 1.0
    } else {
        octaves
    };

    // Apply octave shift using bit shifting for integer octaves
    let octave_multiplier = if adjusted_octaves >= 0.0 {
        (1 << adjusted_octaves as i32) as f32
    } else {
        1.0 / (1 << (-adjusted_octaves as i32)) as f32
    };

    A4_FREQ * ratio * octave_multiplier
}

// // Complete table of frequencies for all MIDI notes 0-127
// const MIDI_FREQS: [f32; 120] = [
//     16.3516, 17.3239, 18.3540, 19.4454, 20.6017, 21.8268, 23.1247, 24.4997, 25.9565, 27.5000,
//     29.1352, 30.8677, // C0 to B0
//     32.7032, 34.6478, 36.7081, 38.8909, 41.2034, 43.6535, 46.2493, 48.9994, 51.9131, 55.0000,
//     58.2705, 61.7354, // C1 to B1
//     65.4064, 69.2957, 73.4162, 77.7817, 82.4069, 87.3071, 92.4986, 97.9989, 103.826, 110.000,
//     116.541, 123.471, // C2 to B2
//     130.813, 138.591, 146.832, 155.563, 164.814, 174.614, 184.997, 195.998, 207.652, 220.000,
//     233.082, 246.942, // C3 to B3
//     261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994, 391.995, 415.305, 440.000,
//     466.164, 493.883, // C4 to B4
//     523.251, 554.365, 587.330, 622.254, 659.255, 698.456, 739.989, 783.991, 830.609, 880.000,
//     932.328, 987.767, // C5 to B5
//     1046.50, 1108.73, 1174.66, 1244.51, 1318.51, 1396.91, 1479.98, 1567.98, 1661.22, 1760.00,
//     1864.66, 1975.53, // C6 to B6
//     2093.00, 2217.46, 2349.32, 2489.02, 2637.02, 2793.83, 2959.96, 3135.96, 3322.44, 3520.00,
//     3729.31, 3951.07, // C7 to B7
//     4186.01, 4434.92, 4698.64, 4978.03, 5274.04, 5587.65, 5919.91, 6271.93, 6644.88, 7040.00,
//     7458.62, 7902.13, // C8 to B8
//     8372.02, 8869.84, 9397.27, 9956.06, 10548.1, 11175.3, 11839.8, 12543.9, 13289.8, 14080.0,
//     14917.2, 15804.3, // C9 to G9
// ];
//
// // /// Converts frequency in Hz to MIDI note number with high precision
// #[inline(always)]
// pub fn note_to_frequency(note: f32) -> f32 {
//     if note <= 0.0 {
//         return MIDI_FREQS[0];
//     }
//     if note >= 119.0 {
//         return MIDI_FREQS[119];
//     }

//     let index = note as usize;
//     let freq = MIDI_FREQS[index];
//     let x = note.fract();
//     if x > 0.0 {
//         let next_freq = MIDI_FREQS[index + 1];
//         lerp(freq, next_freq, x)
//     } else {
//         freq
//     }
// }
