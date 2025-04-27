use tato_video::{color::PaletteID, *};

pub struct TextBundle {
    pub initial_tile: u8,
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub palette: PaletteID,
}

pub fn draw_text(vid: &mut VideoChip, text: &str, bundle: TextBundle) {
    debug_assert!(text.is_ascii());
    let mut offset_x = 0;
    let mut offset_y = 0;
    for ch in text.chars() {
        vid.bg_map.set_tile(BgBundle {
            col: bundle.col + offset_x,
            row: bundle.row + offset_y,
            tile_id: TileID(char_to_id_ex(ch) + bundle.initial_tile),
            flags: bundle.palette.into(),
        });
        offset_x += 1;
        if offset_x == bundle.width {
            offset_x = 0;
            offset_y += 1;
        }
    }
}

/// Extended from the previous functions to include lowercase letters and additional punctuation.
#[allow(dead_code)]
pub fn char_to_id_ex(ch: char) -> u8 {
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
        _ => 0,
    }
}



/// Intended for extremely simple, early 80's arcade text containing
/// only numbers and upper case letters and basic punctuation.
#[allow(dead_code)] // Function is defined but never used
fn char_to_id(ch: char) -> u8 {
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
