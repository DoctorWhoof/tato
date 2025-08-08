use super::*;
use crate::libm;

// Type conversion implementations
impl From<Rect<u8>> for Rect<i32> {
    fn from(val: Rect<u8>) -> Self {
        Rect {
            x: val.x.into(),
            y: val.y.into(),
            w: val.w.into(),
            h: val.h.into(),
        }
    }
}

impl From<Rect<u16>> for Rect<i32> {
    fn from(val: Rect<u16>) -> Self {
        Rect {
            x: val.x.into(),
            y: val.y.into(),
            w: val.w.into(),
            h: val.h.into(),
        }
    }
}

impl From<Rect<i8>> for Rect<i32> {
    fn from(val: Rect<i8>) -> Self {
        Rect {
            x: val.x.into(),
            y: val.y.into(),
            w: val.w.into(),
            h: val.h.into(),
        }
    }
}



impl From<Rect<i32>> for Rect<f32> {
    fn from(rect: Rect<i32>) -> Self {
        Rect {
            x: rect.x as f32,
            y: rect.y as f32,
            w: rect.w as f32,
            h: rect.h as f32,
        }
    }
}

impl From<Rect<f32>> for Rect<i32> {
    fn from(rect: Rect<f32>) -> Self {
        Rect {
            x: libm::floorf(rect.x) as i32,
            y: libm::floorf(rect.y) as i32,
            w: libm::floorf(rect.w) as i32,
            h: libm::floorf(rect.h) as i32,
        }
    }
}

impl From<Rect<i16>> for Rect<f32> {
    fn from(rect: Rect<i16>) -> Self {
        Rect {
            x: rect.x as f32,
            y: rect.y as f32,
            w: rect.w as f32,
            h: rect.h as f32,
        }
    }
}

impl From<Rect<i8>> for Rect<f32> {
    fn from(rect: Rect<i8>) -> Self {
        Rect {
            x: rect.x as f32,
            y: rect.y as f32,
            w: rect.w as f32,
            h: rect.h as f32,
        }
    }
}

impl From<Rect<f32>> for Rect<i16> {
    fn from(rect: Rect<f32>) -> Self {
        Rect {
            x: libm::floorf(rect.x) as i16,
            y: libm::floorf(rect.y) as i16,
            w: libm::floorf(rect.w) as i16,
            h: libm::floorf(rect.h) as i16,
        }
    }
}
