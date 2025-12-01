use tato_video::{COLORS_PER_TILE, SUBPALETTE_COUNT};

pub(crate) struct SubPaletteInsert {
    pub position: usize,
    // Remapping rule:
    // index: original color position
    // value: new color position
    pub remapping: [usize; COLORS_PER_TILE as usize],
}

#[derive(Debug, Default, Clone)]
pub(crate) struct SubPalette {
    indices: [u8; COLORS_PER_TILE as usize],
    count: u8,
}

impl SubPalette {
    pub fn push(&mut self, value: u8) {
        if self.count >= SUBPALETTE_COUNT {
            panic!(
                "\x1b[31mSubPalette Error:\x1b[0m Sub-palette color count exceeds limit of {}!",
                SUBPALETTE_COUNT
            );
        }
        // Avoid pushing existing values
        if !self.colors().contains(&value) {
            self.indices[self.count as usize] = value;
            self.count += 1;
        }
    }

    pub fn colors(&self) -> &[u8] {
        &self.indices[0..self.count as usize]
    }
}

#[derive(Debug, Default)]
pub(crate) struct SubPaletteBuilder {
    data: Vec<SubPalette>,
}

impl SubPaletteBuilder {
    pub fn add(&mut self, incoming: SubPalette) -> SubPaletteInsert {
        // Check for matches
        // Test case: inserting [0,1], into existing:
        // [2,3]

        for (position, candidate) in &mut self.data.iter().enumerate() {
            let mut is_match = true;
            let mut candidate_modified = candidate.clone();
            let mut remapping = [0; COLORS_PER_TILE as usize];

            for (i, color) in incoming.colors().iter().enumerate() {
                if let Some(match_index) = find(*color, candidate_modified.colors()) {
                    // If candidate has the incoming color, store the remap and proceed
                    remapping[i] = match_index;
                    candidate_modified.push(*color);
                } else {
                    // If not check for available spots
                    if candidate.count < COLORS_PER_TILE {
                        remapping[i] = candidate_modified.count as usize;
                        candidate_modified.push(*color);
                    } else {
                        // No other options, make new subpalette
                        is_match = false;
                        break;
                    }
                }
            }

            if is_match == true {
                self.data[position] = candidate_modified;
                return SubPaletteInsert { position, remapping };
            }
        }

        self.insert(incoming)
    }

    // Force-inserts a new subpalette without checking for matches, returns its position
    fn insert(&mut self, subpalette: SubPalette) -> SubPaletteInsert {
        let position = self.data.len();
        if position >= SUBPALETTE_COUNT as usize {
            panic!(
                "\x1b[31mSubPalette Builder Error:\x1b[0m Sub-palette exceeds limit of {}!",
                SUBPALETTE_COUNT
            );
        }
        self.data.push(subpalette);
        SubPaletteInsert { position, remapping: [0, 0, 0, 0] }
    }
}

fn find(value: u8, colors: &[u8]) -> Option<usize> {
    let mut i = 0;
    for color in colors {
        if *color == value {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[test]
fn test_subpalette_inserts() {
    let mut builder = SubPaletteBuilder::default();
    assert_eq!(0, builder.data.len());

    let subpalette_a = SubPalette { indices: [2, 3, 0, 0], count: 2 };
    let insert_a = builder.insert(subpalette_a);
    assert_eq!([0, 0, 0, 0], insert_a.remapping);
    assert_eq!(2, builder.data[0].count);
    assert_eq!(0, insert_a.position);
    assert_eq!(1, builder.data.len());

    let subpalette_b = SubPalette { indices: [0, 1, 0, 0], count: 2 };
    let insert_b = builder.add(subpalette_b);
    assert_eq!(1, builder.data.len());
    assert_eq!(4, builder.data[0].count);
    assert_eq!(0, insert_b.position);
    assert_eq!([2, 3, 0, 0], insert_b.remapping);
    assert_eq!([2, 3, 0, 1], builder.data[0].indices);

    let subpalette_c = SubPalette { indices: [0, 0, 0, 0], count: 1 };
    let insert_c = builder.add(subpalette_c);
    assert_eq!(0, insert_c.position);
    assert_eq!([2, 0, 0, 0], insert_c.remapping);

    let subpalette_d = SubPalette { indices: [0, 4, 5, 6], count: 4 };
    let insert_d = builder.add(subpalette_d);
    assert_eq!(1, insert_d.position);
    assert_eq!([0, 0, 0, 0], insert_d.remapping);

    let subpalette_e = SubPalette { indices: [2, 3, 5, 0], count: 3 };
    let insert_e = builder.add(subpalette_e);
    assert_eq!(2, insert_e.position);
    assert_eq!([0, 0, 0, 0], insert_e.remapping);

    assert_eq!(3, builder.data.len());

    let subpalette_f = SubPalette { indices: [0, 0, 0, 0], count: 1 };
    let insert_f = builder.add(subpalette_f);
    assert_eq!(0, insert_f.position);
    assert_eq!([2, 0, 0, 0], insert_f.remapping);

    let subpalette_g = SubPalette { indices: [5, 0, 0, 0], count: 1 };
    let insert_g = builder.add(subpalette_g);
    assert_eq!(1, insert_g.position);
    assert_eq!([2, 0, 0, 0], insert_g.remapping);
}
