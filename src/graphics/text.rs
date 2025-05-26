use crate::{AnimID, MapID, Tato, TilesetID};
use tato_video::{color::PaletteID, *};

pub struct TextOp {
    pub id: TilesetID,
    pub col: u16,
    pub row: u16,
    pub width: u16,
    pub palette: PaletteID,
}

impl Tato {
    /// "Draws" a text string to the BG Map, returns the resulting height (in rows), if any.
    pub fn draw_text(&mut self, text: &str, op: TextOp) -> Option<u16> {
        debug_assert!(text.is_ascii());
        let tileset = self
            .assets
            .tilesets
            .get(op.id.0 as usize)?;

        // let tileset = &self.tiles.sets[bundle.tileset.0 as usize];
        let mut set_tile = |ch: char, cursor_x: u16, cursor_y: u16| {
            self.banks[tileset.bank_id as usize].bg.set_cell(BgOp {
                col: op.col + cursor_x,
                row: op.row + cursor_y,
                tile_id: TileID(char_to_id_ex(ch) + tileset.tile_start),
                flags: op.palette.into(),
            });
        };

        let mut cursor_x = 0;
        let mut cursor_y = 0;
        for word in text.split(' ') {
            if cursor_x + (word.len() as u16) > op.width {
                cursor_x = 0;
                cursor_y += 1;
            }
            for ch in word.chars() {
                set_tile(ch, cursor_x, cursor_y);
                cursor_x += 1;
            }
            if cursor_x >= op.width {
                cursor_x = 0;
                cursor_y += 1;
            } else {
                set_tile(' ', cursor_x, cursor_y);
                cursor_x += 1;
            }
        }
        // If successful, return number of lines written
        Some(cursor_y + 1)
    }
}

/// Extended from the previous functions to include lowercase letters and additional punctuation.
#[allow(dead_code)]
fn char_to_id_ex(ch: char) -> u16 {
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
