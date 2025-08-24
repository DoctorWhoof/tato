use crate::TilesetID;
use tato_video::{DynTilemap, color::PaletteID};

pub struct TextOp<'a> {
    pub font: &'a dyn DynTilemap, // Can be &Tilemap or TilemapRef!
    pub id: TilesetID,
    pub col: i16,
    pub row: i16,
    pub width: i16,
    pub palette: PaletteID,
}

/// Extended from the previous functions to include lowercase letters and additional punctuation.
#[allow(dead_code)]
pub(crate) fn char_to_id_ex(ch: char) -> u8 {
    match ch {
        // Tightly packed ASCII chars in their original order can
        // result in fast optimizations by the compiler
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        // These are also contiguous
        'A' => 10,
        'B' => 11,
        'C' => 12,
        'D' => 13,
        'E' => 14,
        'F' => 15,
        'G' => 16,
        'H' => 17,
        'I' => 18,
        'J' => 19,
        'K' => 20,
        'L' => 21,
        'M' => 22,
        'N' => 23,
        'O' => 24,
        'P' => 25,
        'Q' => 26,
        'R' => 27,
        'S' => 28,
        'T' => 29,
        'U' => 30,
        'V' => 31,
        'W' => 32,
        'X' => 33,
        'Y' => 34,
        'Z' => 35,
        // Lowercase letters immediately after uppercase
        'a' => 36,
        'b' => 37,
        'c' => 38,
        'd' => 39,
        'e' => 40,
        'f' => 41,
        'g' => 42,
        'h' => 43,
        'i' => 44,
        'j' => 45,
        'k' => 46,
        'l' => 47,
        'm' => 48,
        'n' => 49,
        'o' => 50,
        'p' => 51,
        'q' => 52,
        'r' => 53,
        's' => 54,
        't' => 55,
        'u' => 56,
        'v' => 57,
        'w' => 58,
        'x' => 59,
        'y' => 60,
        'z' => 61,
        // These are also contiguous
        ':' => 62,
        ';' => 63,
        '<' => 64,
        '=' => 65,
        '>' => 66,
        '?' => 67,
        // And so are these
        ' ' => 68,
        '!' => 69,
        '"' => 70,
        '#' => 71,
        '$' => 72,
        '%' => 73,
        '&' => 74,
        '\'' => 75,
        '(' => 76,
        ')' => 77,
        '*' => 78,
        '+' => 79,
        ',' => 80,
        '-' => 81,
        '.' => 82,
        '/' => 83,
        // But not these
        '_' => 84,
        _ => 0,
    }
}

/// Intended for extremely simple, early 80's arcade text containing
/// only numbers and upper case letters and basic punctuation.
#[allow(dead_code)] // Function is defined but never used
pub(crate) fn char_to_id(ch: char) -> u8 {
    match ch {
        // Tightly packed ASCII chars in their original order can
        // result in fast optimizations by the compiler
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        // These are also contiguous
        'A' => 10,
        'B' => 11,
        'C' => 12,
        'D' => 13,
        'E' => 14,
        'F' => 15,
        'G' => 16,
        'H' => 17,
        'I' => 18,
        'J' => 19,
        'K' => 20,
        'L' => 21,
        'M' => 22,
        'N' => 23,
        'O' => 24,
        'P' => 25,
        'Q' => 26,
        'R' => 27,
        'S' => 28,
        'T' => 29,
        'U' => 30,
        'V' => 31,
        'W' => 32,
        'X' => 33,
        'Y' => 34,
        'Z' => 35,
        // These aren't contiguous, but I want to keep it simple
        '?' => 36,
        ',' => 37,
        '.' => 38,
        ' ' => 39,
        _ => 0,
    }
}
