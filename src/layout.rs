use std::hash::Hash;

use crate::*;
use Direction::*;

pub struct Layout<ID>
where
    ID: Hash + Eq,
{
    pub width: u16,
    pub height: u16,
    pub scale: f32,
    pub rects: HashMap<ID, Rect>,
    pub frames: Vec<Frame>,
    head: usize,
    // pub root: Frame,
}

struct Frame {
    direction: Direction,
    start: u16,
    end: u16,
    rect: Rect,
}

impl<ID> Default for Layout<ID>
where
    ID: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<ID> Layout<ID>
where
    ID: Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            width: 100,
            height: 100,
            scale: 1.0,
            rects: HashMap::new(),
            frames: Vec::new(),
            head: 0,
            // root: Frame { start: (), end: (), direction: (), margin: (), gap: (), children: () },
        }
    }

    pub fn width(self, width: u16) -> Self {
        Self { width, ..self }
    }

    pub fn height(self, height: u16) -> Self {
        Self { height, ..self }
    }

    pub fn horizontal(mut self) -> Self {
        self.head = self.frames.len();
        let rect = if self.head == 0 {
            Rect {
                x: 0,
                y: 0,
                w: self.width,
                h: self.height,
            }
        } else {
            let container = self.frames.get(self.head).unwrap();
            Rect {
                x: 0,
                y: 0,
                w: self.width,
                h: self.height,
            }
        };
        self.frames.push(Frame {
            direction: Horizontal,
            start: 0,
            end: 0,
            rect,
        });
        self
    }

    pub fn push_left(mut self, size: u16, id: ID) -> Self {
        if let Some(parent) = self.frames.get_mut(self.head) {
            assert!(!self.rects.contains_key(&id), "Layout Error: Redundant key");
            parent.start += size;
            self.rects.insert(
                id,
                Rect {
                    x: parent.rect.x + parent.start,
                    y: parent.rect.y,
                    w: size,
                    h: parent.rect.h,
                },
            );
        } else {
            panic!("Layout Error: Cannot acquire parent container");
        }
        self
    }

    // pub fn push_end(mut self, size: u16, id: ID) -> Self {
    //     if let Some(container) = self.frames.get_mut(self.head) {
    //         container.end += size;
    //     } else {
    //         panic!("Layout: Error acquiring container");
    //     }
    //     self
    // }
}
