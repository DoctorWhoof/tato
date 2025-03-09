#![no_std]

// TODO: Determine how much rects are overlapping if Layout is too small, and shrink each one accordingly
// OR return result with difference

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub rect: Rect,
    pub cursor: Rect,
    pub margin: u16,
    pub scale: f32,
}

#[derive(Debug, Clone, Copy)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

impl Frame {
    pub fn new(rect: Rect, margin: u16) -> Self {
        let scale = 1.0;
        let margin = (margin as f32 * scale) as u16;
        let cursor = rect_shrink(rect, margin);
        Self {
            rect,
            cursor,
            margin,
            scale,
        }
    }

    // "scale" is required since "fill" methods always pass a scale of 1.0,
    // while "add" methods use the scale field.
    fn add(&mut self, side: Side, len: u16, scale: f32, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * scale) as u16;
        let margin = (self.margin as f32 * self.scale) as u16;

        // Calculate the child rectangle based on the side
        let child_rect = match side {
            Side::Left => Rect {
                x: self.cursor.x,
                y: self.cursor.y,
                w: scaled_len,
                h: self.cursor.h,
            },
            Side::Right => Rect {
                x: (self.cursor.x + self.cursor.w).saturating_sub(scaled_len),
                y: self.cursor.y,
                w: scaled_len,
                h: self.cursor.h,
            },
            Side::Top => Rect {
                x: self.cursor.x,
                y: self.cursor.y,
                w: self.cursor.w,
                h: scaled_len,
            },
            Side::Bottom => Rect {
                x: self.cursor.x,
                y: (self.cursor.y + self.cursor.h).saturating_sub(scaled_len),
                w: self.cursor.w,
                h: scaled_len,
            },
        };

        let child_cursor = rect_shrink(child_rect, margin);

        // Check if the child fits
        let is_horizontal = matches!(side, Side::Left | Side::Right);
        let dimension = if is_horizontal {
            child_rect.w
        } else {
            child_rect.h
        };
        let parent_dimension = if is_horizontal {
            self.cursor.w
        } else {
            self.cursor.h
        };
        if dimension > parent_dimension || dimension < margin {
            return;
        }

        // Update parent cursor
        match side {
            Side::Left => {
                self.cursor.x += scaled_len + margin;
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + margin);
            }
            Side::Right => {
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + margin);
            }
            Side::Top => {
                self.cursor.y += scaled_len + margin;
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + margin);
            }
            Side::Bottom => {
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + margin);
            }
        }

        // Call the function with the new frame
        func(&mut Frame {
            rect: child_rect,
            cursor: child_cursor,
            margin: self.margin,
            scale: self.scale,
        })
    }

    #[inline(always)]
    pub fn add_left(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Left, len, self.scale, func)
    }

    #[inline(always)]
    pub fn add_right(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Right, len, self.scale, func)
    }

    #[inline(always)]
    pub fn add_top(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Top, len, self.scale, func)
    }

    #[inline(always)]
    pub fn add_bottom(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Bottom, len, self.scale, func)
    }

    // Fill methods can also be unified
    fn fill(&mut self, side: Side, ratio: f32, func: impl FnMut(&mut Frame)) {
        let is_horizontal = matches!(side, Side::Left | Side::Right);
        let len = if is_horizontal {
            self.cursor.w as f32 * ratio.clamp(0.0, 1.0)
        } else {
            self.cursor.h as f32 * ratio.clamp(0.0, 1.0)
        };

        self.add(side, len as u16, 1.0, func);
    }

    #[inline(always)]
    pub fn fill_left(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Left, ratio, func);
    }

    #[inline(always)]
    pub fn fill_top(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Top, ratio, func);
    }

    #[inline(always)]
    pub fn fill_right(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Right, ratio, func);
    }

    #[inline(always)]
    pub fn fill_bottom(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Bottom, ratio, func);
    }
}

#[inline(always)]
fn rect_shrink(rect: Rect, margin: u16) -> Rect {
    Rect {
        x: rect.x + margin,
        y: rect.y + margin,
        w: rect.w.saturating_sub(margin * 2),
        h: rect.h.saturating_sub(margin * 2),
    }
}
