use crate::math::note_to_frequency;

/// The Note enum can be used in lieu of MIDI note codes in any function
/// that takes i32 or f32 as an argument, via the "into()" method.

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Note {
    C0 = 12, CSharp0, D0, DSharp0, E0, F0, FSharp0, G0, GSharp0, A0, ASharp0, B0,
    C1, CSharp1, D1, DSharp1, E1, F1, FSharp1, G1, GSharp1, A1, ASharp1, B1,
    C2, CSharp2, D2, DSharp2, E2, F2, FSharp2, G2, GSharp2, A2, ASharp2, B2,
    C3, CSharp3, D3, DSharp3, E3, F3, FSharp3, G3, GSharp3, A3, ASharp3, B3,
    C4, CSharp4, D4, DSharp4, E4, F4, FSharp4, G4, GSharp4, A4, ASharp4, B4,
    C5, CSharp5, D5, DSharp5, E5, F5, FSharp5, G5, GSharp5, A5, ASharp5, B5,
    C6, CSharp6, D6, DSharp6, E6, F6, FSharp6, G6, GSharp6, A6, ASharp6, B6,
    C7, CSharp7, D7, DSharp7, E7, F7, FSharp7, G7, GSharp7, A7, ASharp7, B7,
    C8, CSharp8, D8, DSharp8, E8, F8, FSharp8, G8, GSharp8, A8, ASharp8, B8,
    C9, CSharp9, D9, DSharp9, E9, F9, FSharp9, G9, GSharp9, A9, ASharp9, B9,
    C10,
}

impl Note {
    /// Corresponding frequency in Hz.
    pub fn frequency(&self) -> f32 {
        note_to_frequency((*self as i32) as f32)
    }

    /// Corresponding MIDI note.
    pub fn midi_note(&self) -> f32 {
        (*self as i32) as f32
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
