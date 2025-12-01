use std::{array::from_fn, collections::HashMap};

use tato_video::{COLORS_PER_TILE, SUBPALETTE_COUNT};

pub(crate) struct SubPaletteInsert {
    pub position: u8,
    pub mapping: HashMap<u8, u8>,
    // key: source color index (main palette)
    // value: index in the final subpalette
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub(crate) struct SubPalette {
    colors: [u8; COLORS_PER_TILE as usize],
    count: u8,
}

impl SubPalette {
    pub fn from(source: &[u8]) -> Self {
        Self {
            colors: from_fn(|i| if i < source.len() { source[i] } else { 0 }),
            count: source.len() as u8,
        }
    }

    pub fn generate_mapping(&self) -> HashMap<u8, u8> {
        let mut result = HashMap::new();
        // populate mapping with existing colors
        for (i, &color) in self.colors().iter().enumerate() {
            result.insert(color, i as u8);
        }
        result
    }

    pub fn push(&mut self, value: u8) {
        // Avoid pushing existing values
        if !self.colors().contains(&value) {
            if self.count >= COLORS_PER_TILE {
                panic!(
                    "\x1b[31mSubPalette Error:\x1b[0m Sub-palette color count exceeds limit of {}!",
                    COLORS_PER_TILE
                );
            }
            self.colors[self.count as usize] = value;
            self.count += 1;
        }
    }

    pub fn colors(&self) -> &[u8] {
        &self.colors[0..self.count as usize]
    }
}

#[derive(Debug, Default)]
pub(crate) struct SubPaletteBuilder {
    map: HashMap<SubPalette, u8>,
    data: Vec<SubPalette>,
}

impl SubPaletteBuilder {
    pub fn data(&self) -> &[SubPalette] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn add_new(&mut self, incoming: SubPalette) -> SubPaletteInsert {
        for (position, candidate) in &mut self.data.iter_mut().enumerate() {
            let mut is_match = true;
            let mut mapping = incoming.generate_mapping();

            for &color in incoming.colors() {
                let candidate_mapping = candidate.generate_mapping();
                if let Some(match_index) = find(color, candidate.colors()) {
                    // Color already exists in candidate
                    if candidate_mapping.get(&color) == mapping.get(&color) {
                        mapping.insert(color, match_index as u8);
                    } else {
                        is_match = false;
                        break;
                    }
                } else {
                    // Add new color
                    if candidate.count < COLORS_PER_TILE {
                        if candidate_mapping.get(&color) == mapping.get(&color) {
                            mapping.insert(color, candidate.count);
                            candidate.push(color);
                        } else {
                            is_match = false;
                            break;
                        }
                    } else {
                        is_match = false;
                        break;
                    }
                }
            }

            if is_match {
                self.map.insert(incoming, position as u8);
                return SubPaletteInsert { position: position as u8, mapping };
            }
        }

        self.insert(incoming)
    }

    pub fn add(&mut self, incoming: SubPalette) -> SubPaletteInsert {
        // Early return
        if let Some(i) = self.map.get(&incoming) {
            let entry = &self.data[*i as usize];
            return SubPaletteInsert { position: *i, mapping: entry.generate_mapping() };
        }
        // Search for matches
        for (position, candidate) in &mut self.data.iter_mut().enumerate() {
            let mut is_match = true;
            let mut mapping = incoming.generate_mapping();

            for &color in incoming.colors() {
                if let Some(match_index) = find(color, candidate.colors()) {
                    // Color already exists in candidate
                    mapping.insert(color, match_index as u8);
                } else {
                    // Add new color
                    if candidate.count < COLORS_PER_TILE {
                        mapping.insert(color, candidate.count);
                        candidate.push(color);
                    } else {
                        is_match = false;
                        break;
                    }
                }
            }

            if is_match {
                self.map.insert(incoming, position as u8);
                return SubPaletteInsert { position: position as u8, mapping };
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
        let mapping = subpalette.generate_mapping();
        self.map.insert(subpalette.clone(), position as u8);
        self.data.push(subpalette);
        SubPaletteInsert { position: position as u8, mapping }
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

    let subpalette_a = SubPalette { colors: [2, 3, 0, 0], count: 2 };
    let insert_a = builder.insert(subpalette_a);
    assert_eq!(None, insert_a.mapping.get(&1));
    assert_eq!(Some(&0), insert_a.mapping.get(&2));
    assert_eq!(Some(&1), insert_a.mapping.get(&3));
    assert_eq!(2, builder.data[0].count);
    assert_eq!(0, insert_a.position);
    assert_eq!(1, builder.data.len());

    let subpalette_b = SubPalette { colors: [0, 1, 0, 0], count: 2 };
    let insert_b = builder.add(subpalette_b);
    assert_eq!(1, builder.data.len());
    assert_eq!(4, builder.data[0].count);
    assert_eq!(0, insert_b.position);
    assert_eq!(None, insert_b.mapping.get(&5));
    assert_eq!(Some(&2), insert_b.mapping.get(&0));
    assert_eq!(Some(&3), insert_b.mapping.get(&1));
    assert_eq!([2, 3, 0, 1], builder.data[0].colors);

    let subpalette_c = SubPalette { colors: [0, 0, 0, 0], count: 1 };
    let insert_c = builder.add(subpalette_c);
    assert_eq!(0, insert_c.position);
    // assert_eq!([2, 0, 0, 0], insert_c.mapping);

    let subpalette_d = SubPalette { colors: [0, 4, 5, 6], count: 4 };
    let insert_d = builder.add(subpalette_d);
    assert_eq!(1, insert_d.position);
    // assert_eq!([0, 0, 0, 0], insert_d.mapping);

    let subpalette_e = SubPalette { colors: [2, 3, 5, 0], count: 3 };
    let insert_e = builder.add(subpalette_e);
    assert_eq!(2, insert_e.position);
    // assert_eq!([0, 0, 0, 0], insert_e.mapping);

    assert_eq!(3, builder.data.len());

    let subpalette_f = SubPalette { colors: [0, 0, 0, 0], count: 1 };
    let insert_f = builder.add(subpalette_f);
    assert_eq!(0, insert_f.position);
    // assert_eq!([2, 0, 0, 0], insert_f.mapping);

    let subpalette_g = SubPalette { colors: [5, 0, 0, 0], count: 1 };
    let insert_g = builder.add(subpalette_g);
    assert_eq!(1, insert_g.position);
    // assert_eq!([2, 0, 0, 0], insert_g.mapping);
}
