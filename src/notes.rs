// Pre-computed table of the 12 semitone ratios within an octave
const SEMITONE_RATIOS: [f32; 12] = [
    1.0, 1.059463, 1.122462, 1.189207, 1.259921, 1.334840, 1.414214, 1.498307, 1.587401, 1.681793,
    1.781797, 1.887749,
];

// Pre-computed table of frequency ratios for each semitone relative to the base note
// These are approximately the inverses of the semitone ratios used in note_to_frequency
const SEMITONE_THRESHOLDS: [f32; 12] = [
    1.0,      // C (ratio = 1.0)
    1.029302, // C#/Db (halfway between 1.0 and 1.059463)
    1.090507, // D  (halfway between 1.059463 and 1.122462)
    1.155334, // D#/Eb
    1.223040, // E
    1.296840, // F
    1.376900, // F#/Gb
    1.461700, // G
    1.551300, // G#/Ab
    1.646090, // A
    1.747455, // A#/Bb
    1.853020, // B (halfway between 1.781797 and 1.887749)
];

// Wraps a value into a range from 0 to modulus, correctly handling negative numbers.
#[inline(always)]
fn wrap(value: i32, modulus: i32) -> i32 {
    ((value % modulus) + modulus) % modulus
}

/// Returns the MIDI note value given an octave (zero to 10) and a note (zero to 11).
#[inline(always)]
pub fn get_midi_note(octave: impl Into<i32>, note: impl Into<i32>) -> i32 {
    // Handle negative values and values beyond range
    let octave = wrap(octave.into(), 10);
    let note = wrap(note.into(), 12);
    // MIDI note number, where C4 is 60
    ((octave + 1) * 12) + note
}

/// The corresponding MIDI note of a frequency in Hz
#[inline(always)]
pub fn frequency_to_note(frequency: f32) -> f32 {
    // Find ratio compared to A4 (440Hz)
    let ratio = frequency / 440.0;

    // Find octave
    let mut octaves = 0;
    let mut adjusted_ratio = ratio;

    // Handle ratio > 2.0 (higher octaves)
    while adjusted_ratio >= 2.0 {
        adjusted_ratio /= 2.0;
        octaves += 1;
    }

    // Handle ratio < 1.0 (lower octaves)
    while adjusted_ratio < 1.0 {
        adjusted_ratio *= 2.0;
        octaves -= 1;
    }

    // Now adjusted_ratio is between 1.0 and 2.0
    // Find which semitone this ratio corresponds to
    let mut semitone = 0;
    while semitone < 11 && adjusted_ratio >= SEMITONE_THRESHOLDS[semitone + 1] {
        semitone += 1;
    }

    // The note value is A4 (69) plus the octave shift in semitones (12 per octave)
    // plus the semitone index within the octave
    69.0 + (octaves * 12) as f32 + semitone as f32
}

#[inline(always)]
/// The frequency in Hz of any MIDI note value.
// This was much more difficult to do without the standard library... :-P
pub fn note_to_frequency(note: f32) -> f32 {
    // Calculate how many semitones from A4 (440Hz, MIDI note 69)
    let semitones_from_a4 = note - 69.0;

    // Handle the integer part
    let offset_semitones = (semitones_from_a4 + 1200.0) as i32; // Add 100 octaves to ensure positive
    let octave_shift = (offset_semitones / 12) - 100; // Subtract the 100 octaves we added
    let semitone_index = offset_semitones % 12;

    // Get the semitone ratio from our table
    let semitone_ratio = SEMITONE_RATIOS[semitone_index as usize];

    // Calculate base frequency with octave shift using bit shifting for powers of 2
    let octave_multiplier = if octave_shift >= 0 {
        // 2^octave_shift
        (1 << octave_shift) as f32
    } else {
        // Use 1.0/(2^|octave_shift|)
        1.0 / (1 << (-octave_shift)) as f32
    };

    440.0 * semitone_ratio * octave_multiplier
}
