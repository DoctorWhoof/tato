#![no_std]

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub scale: f32,
    pub rect: Rect,
    pub margin: Option<u16>,
}

const MARGIN: u16 = 10;

// TODO: Determine how much rects are overlapping if Layout is too small, and shrink each one accordingly
// OR return result with difference
impl Frame {
    pub fn root(x: u16, y: u16, w: u16, h: u16) -> Self {
        let scale = 1.0;
        let rect = Rect { x, y, w, h }; // Unaffected by scale! All child frames are, though.
        Self {
            rect,
            scale,
            margin: Some(MARGIN),
        }
    }

    pub fn add_left(&mut self, len: u16, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * self.scale) as u16;
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let h = self.rect.h.saturating_sub(margin * 2);
        let child = Rect {
            x: self.rect.x + margin,
            y: self.rect.y + margin,
            w: scaled_len,
            h,
        };
        if child.w + margin > self.rect.w || child.w < (MARGIN * 2) as u16 {
            return;
        }
        self.rect.x += scaled_len + margin;
        self.rect.w = self.rect.w.saturating_sub(scaled_len + margin);

        func(&mut Frame {
            scale: self.scale,
            rect: child,
            margin: self.margin,
        })
    }

    pub fn add_right(&mut self, len: u16, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * self.scale) as u16;
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let child = Rect {
            x: (self.rect.x + self.rect.w)
                .saturating_sub(margin)
                .saturating_sub(scaled_len),
            y: self.rect.y + margin,
            w: scaled_len,
            h: self.rect.h.saturating_sub(margin * 2),
        };
        if child.w + margin > self.rect.w || child.w < (MARGIN * 2) as u16 {
            return;
        }

        self.rect.w = self.rect.w.saturating_sub(scaled_len + margin);

        func(&mut Frame {
            scale: self.scale,
            rect: child,
            margin: self.margin,
        })
    }

    pub fn add_top(&mut self, len: u16, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * self.scale) as u16;
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let w = self.rect.w.saturating_sub(margin * 2);
        let child = Rect {
            x: self.rect.x + margin,
            y: self.rect.y + margin,
            w,
            h: scaled_len,
        };
        if child.h + margin > self.rect.h || child.h < (MARGIN * 2) as u16 {
            return;
        }
        self.rect.y += scaled_len + margin;
        self.rect.h = self.rect.h.saturating_sub(scaled_len + margin);

        func(&mut Frame {
            scale: self.scale,
            rect: child,
            margin: self.margin,
        })
    }

    pub fn add_bottom(&mut self, len: u16, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * self.scale) as u16;
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let child = Rect {
            x: self.rect.x + margin,
            y: (self.rect.y + self.rect.h)
                .saturating_sub(margin)
                .saturating_sub(scaled_len),
            w: self.rect.w.saturating_sub(margin * 2),
            h: scaled_len,
        };
        if child.h + margin > self.rect.h || child.h < (MARGIN * 2) as u16 {
            return;
        }

        self.rect.h = self.rect.h.saturating_sub(scaled_len + margin);

        func(&mut Frame {
            scale: self.scale,
            rect: child,
            margin: self.margin,
        })
    }

    pub fn fill_left(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let len_f32 = (self.rect.w as f32 * ratio.clamp(0.0, 1.0)) - (margin * 2) as f32;
        let len = (len_f32.clamp(0.0, u16::MAX as f32) / self.scale) as u16;
        self.add_left(len, func);
    }

    pub fn fill_top(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        let margin = (self.margin.unwrap_or(MARGIN) as f32 * self.scale) as u16;
        let len_f32 = (self.rect.h as f32 * ratio.clamp(0.0, 1.0)) - (margin * 2) as f32;
        let len = (len_f32.clamp(0.0, u16::MAX as f32) / self.scale) as u16;
        self.add_top(len, func);
    }
}
