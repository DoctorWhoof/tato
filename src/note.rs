use crate::math::{get_midi_note, note_to_frequency};

/// The Note enum can be used in lieu of MIDI note codes in any function
/// that takes i32 or f32 as an argument, via the "into()" method.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B
}

impl Note {
    /// Obtains the corresponding frequency in Hz.
    pub fn frequency(&self, octave:i32) -> f32 {
        let midi_note = get_midi_note(octave, *self) ;
        note_to_frequency(midi_note as f32)
    }
}

impl Into<i32> for Note {
    fn into(self) -> i32 {
        self as i32
    }
}

impl Into<f32> for Note {
    fn into(self) -> f32 {
        (self as u8) as f32
    }
}
