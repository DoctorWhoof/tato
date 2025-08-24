use crate::{Float, Num, SignedNum, Vec2};
use core::ops::{Add, Mul, Sub};

mod convert;
mod ops;

/// A generic rectangular area.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T>
where
    T: Num,
{
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect { x, y, w, h }
    }

    pub fn offset(self, delta_x: T, delta_y: T) -> Self {
        Rect { x: self.x + delta_x, y: self.y + delta_y, ..self }
    }

    pub fn pos(&self) -> Vec2<T> {
        Vec2 { x: self.x, y: self.y }
    }

    pub fn size(&self) -> Vec2<T> {
        Vec2 { x: self.w, y: self.h }
    }

    pub fn left(self) -> T {
        self.x
    }

    pub fn right(&self) -> T {
        self.x + self.w
    }

    pub fn top(&self) -> T {
        self.y
    }

    pub fn bottom(&self) -> T {
        self.y + self.h
    }

    pub fn top_left(&self) -> Vec2<T> {
        Vec2 { x: self.x, y: self.y }
    }

    pub fn top_right(&self) -> Vec2<T> {
        Vec2 { x: self.right(), y: self.y }
    }

    pub fn bottom_left(&self) -> Vec2<T> {
        Vec2 { x: self.x, y: self.bottom() }
    }

    pub fn bottom_right(&self) -> Vec2<T> {
        Vec2 { x: self.x + self.w, y: self.y + self.h }
    }

    pub fn center(&self) -> Vec2<T> {
        let two: T = T::one() + T::one();
        Vec2 { x: self.x + (self.w / two), y: self.y + (self.h / two) }
    }

    pub fn intersect(&self, other: Self) -> Option<Self> {
        // Basic overlap check using same logic as overlaps() method (inclusive boundaries)
        if other.x > self.x + self.w {
            return None;
        }
        if other.y > self.y + self.h {
            return None;
        }
        if other.x + other.w < self.x {
            return None;
        }
        if other.y + other.h < self.y {
            return None;
        }

        let x = if self.x > other.x { self.x } else { other.x };
        let y = if self.y > other.y { self.y } else { other.y };
        let right = if self.right() < other.right() { self.right() } else { other.right() };
        let bottom = if self.bottom() < other.bottom() { self.bottom() } else { other.bottom() };
        Some(Rect { x, y, w: right - x, h: bottom - y })
    }

    pub fn clamp_vec(self, vec: Vec2<T>) -> Vec2<T> {
        Vec2 {
            x: if vec.x < self.x {
                self.x
            } else if vec.x > self.right() {
                self.right()
            } else {
                vec.x
            },
            y: if vec.y < self.y {
                self.y
            } else if vec.y > self.bottom() {
                self.bottom()
            } else {
                vec.y
            },
        }
    }

    pub fn shrink(self, margin: T) -> Self {
        if margin == T::zero() {
            return self;
        }
        Self {
            x: self.x + margin,
            y: self.y + margin,
            w: T::get_max(self.w - (margin * T::two()), T::zero()),
            h: T::get_max(self.h - (margin * T::two()), T::zero()),
        }
    }

    pub fn expand(self, margin: T) -> Self {
        if margin == T::zero() {
            return self;
        }
        Self {
            x: self.x - margin,
            y: self.y - margin,
            w: self.w + margin * T::two(),
            h: self.h + margin * T::two(),
        }
    }

    /// Convert this rectangle to a floating-point rectangle for precise calculations
    pub fn to_f32(self) -> Rect<f32> {
        Rect {
            x: self.x.to_f32(),
            y: self.y.to_f32(),
            w: self.w.to_f32(),
            h: self.h.to_f32(),
        }
    }

    /// Convert a floating-point rectangle to this rectangle's number type
    pub fn from_f32(rect: Rect<f32>) -> Self {
        Self {
            x: T::from_f32(rect.x),
            y: T::from_f32(rect.y),
            w: T::from_f32(rect.w),
            h: T::from_f32(rect.h),
        }
    }

    pub fn contains(&self, x: T, y: T) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let self_right = self.x + self.w;
        let self_bottom = self.y + self.h;
        let other_right = other.x + other.w;
        let other_bottom = other.y + other.h;

        self.x <= other_right
            && other.x <= self_right
            && self.y <= other_bottom
            && other.y <= self_bottom
    }
}

impl<T> Rect<T>
where
    T: SignedNum,
{
    pub fn sweep_x(self, delta: T) -> Rect<T>
    where
        T: SignedNum,
    {
        if delta > T::zero() {
            Rect { w: self.w + delta, ..self }
        } else {
            Rect { x: self.x + delta, w: self.w + (-delta), ..self }
        }
    }

    pub fn sweep_y(self, delta: T) -> Rect<T>
    where
        T: SignedNum,
    {
        if delta > T::zero() {
            Rect { h: self.h + delta, ..self }
        } else {
            Rect { y: self.y + delta, h: self.h + (-delta), ..self }
        }
    }
}

// Float-specific methods
impl<T> Rect<T>
where
    T: Float,
{
    /// Linear interpolation between two rectangles
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            w: self.w + (other.w - self.w) * t,
            h: self.h + (other.h - self.h) * t,
        }
    }
}
