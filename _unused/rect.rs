/// A rectangle with its origin at the upper-left corner!
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i16,
    pub y: i16,
    w: u16,
    h: u16,
    piv_x: u16,
    piv_y: u16,
    centered: bool,
}

impl Rect {
    pub fn new(x: i16, y: i16, w: u16, h: u16, centered: bool) -> Self {
        Self {
            x,
            y,
            w,
            h,
            piv_x: if centered { w / 2 } else { 0 },
            piv_y: if centered { h / 2 } else { 0 },
            centered,
        }
    }

    pub fn contains(&self, x: i16, y: i16) -> bool {
        if x < self.left() {
            return false;
        }
        if y < self.top() {
            return false;
        }
        if x >= self.right() {
            return false;
        }
        if y >= self.bottom() {
            return false;
        }
        true
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if other.left() > self.right() {
            return false;
        }
        if other.top() > self.bottom() {
            return false;
        }
        if other.right() < self.left() {
            return false;
        }
        if other.bottom() < self.top() {
            return false;
        }
        true
    }

    pub fn set_width(&mut self, w: u16) {
        self.w = w;
        if self.centered {
            self.piv_x = w / 2
        } else {
            self.piv_x = 0
        }
    }

    pub fn set_height(&mut self, h: u16) {
        self.h = h;
        if self.centered {
            self.piv_y = h / 2
        } else {
            self.piv_y = 0
        }
    }

    #[inline]
    pub fn h(&self) -> u16 {
        self.h
    }

    #[inline]
    pub fn w(&self) -> u16 {
        self.w
    }

    #[inline]
    pub fn left(&self) -> i16 {
        self.x - self.piv_x as i16
    }

    #[inline]
    pub fn top(&self) -> i16 {
        self.y - self.piv_y as i16
    }

    #[inline]
    pub fn right(&self) -> i16 {
        self.x + self.w as i16 - self.piv_x as i16
    }

    #[inline]
    pub fn bottom(&self) -> i16 {
        self.y + self.h as i16 - self.piv_y as i16
    }

    #[inline]
    pub fn pivot_x(&self) -> u16 {
        self.piv_x
    }

    #[inline]
    pub fn pivot_y(&self) -> u16 {
        self.piv_y
    }
}

impl Into<tato_layout::Rect<f32>> for Rect {
    fn into(self) -> tato_layout::Rect<f32> {
        tato_layout::Rect {
            x: self.x as f32,
            y: self.y as f32,
            w: self.w as f32,
            h: self.h as f32,
        }
    }
}

impl From<tato_layout::Rect<f32>> for Rect {
    fn from(value: tato_layout::Rect<f32>) -> Self {
        Self {
            x: value.x as i16,
            y: value.y as i16,
            w: value.w as u16,
            h: value.h as u16,
            piv_x: 0,
            piv_y: 0,
            centered: false,
        }
    }
}
