use super::*;
use core::ops::{Add, Mul, Sub};
use core::fmt::Display;
use num_traits::Num;

/// A generic rectangular area.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rect<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rect<T>
where T: Num + PartialOrd + Copy + Display,
{
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect { x, y, w, h }
    }

    pub fn pos(&self) -> Vec2<T> {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn right(&self) -> T {
        self.x + self.w
    }

    pub fn bottom(&self) -> T {
        self.y + self.h
    }

    pub fn bottom_right(&self) -> Vec2<T> {
        Vec2{
            x: self.x + self.w,
            y: self.y + self.h
        }
    }

    pub fn center(&self) -> Vec2<T> {
        let two:T = T::one() + T::one();
        Vec2 {
            x: self.x + (self.w / two),
            y: self.y + (self.h / two),
        }
    }

    pub fn contains(&self, x: T, y: T) -> bool {
        if x < self.x { return false }
        if y < self.y { return false }
        if x > self.x + self.w { return false }
        if y > self.y + self.h { return false }
        true
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if other.x > self.x + self.w { return false }
        if other.y > self.y + self.h { return false }
        if other.x + other.w < self.x { return false }
        if other.y + other.h < self.y { return false }
        true
    }

    pub fn intersect(&self, other: Self) -> Option<Self> {
        if !self.overlaps(&other) {
            return None;
        }
        let x = if self.x > other.x { self.x } else { other.x };
        let y = if self.y > other.y { self.y } else { other.y };
        let right = if self.right() < other.right() { self.right() } else { other.right() };
        let bottom = if self.bottom() < other.bottom() { self.bottom() } else { other.bottom() };
        Some(Rect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    }
}


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

impl Rect<f32> {
    pub fn to_i32(self) -> Rect<i32> {
        let pos = self.pos().to_i32();
        Rect {
            x: pos.x,
            y: pos.y,
            w: self.w.floor() as i32,
            h: self.h.floor() as i32,
        }
    }
}

// Add/Sub a a rect's position to another rect's position
impl<T> Add<Rect<T>> for Rect<T>
where
    T: Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
{
    type Output = Self;

    fn add(self, other: Rect<T>) -> Self::Output {
        Rect {
            x: self.x + other.x,
            y: self.y + other.y,
            w: self.w,
            h: self.h,
        }
    }
}

impl<T> Sub<Rect<T>> for Rect<T>
where
    T: Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
{
    type Output = Self;

    fn sub(self, other: Rect<T>) -> Self::Output {
        Rect {
            x: self.x - other.x,
            y: self.y - other.y,
            w: self.w,
            h: self.h,
        }
    }
}

// Add/Sub a position vector from rect's position
impl<T, V> Add<Vec2<V>> for Rect<T>
where
    T: Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
    V: Into<T>,
{
    type Output = Self;

    fn add(self, other: Vec2<V>) -> Self::Output {
        Rect {
            x: self.x + other.x.into(),
            y: self.y + other.y.into(),
            w: self.w,
            h: self.h,
        }
    }
}

impl<T, V> Sub<Vec2<V>> for Rect<T>
where
    T: Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
    V: Into<T>,
{
    type Output = Self;

    fn sub(self, other: Vec2<V>) -> Self::Output {
        Rect {
            x: self.x - other.x.into(),
            y: self.y - other.y.into(),
            w: self.w,
            h: self.h,
        }
    }
}

// Multiply by T
impl<T> Mul<T> for Rect<T>
where
    T: Mul<Output = T> + Copy + PartialOrd,
{
    type Output = Self;

    fn mul(self, other: T) -> Self::Output {
        Rect {
            x: self.x * other,
            y: self.y * other,
            w: self.w,
            h: self.h,
        }
    }
}
