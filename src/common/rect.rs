use super::*;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use libm::floorf;
use num_traits::{Float, Num};

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
    T: Num + PartialOrd + MinMax + Copy,
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
        if x < self.x {
            return false;
        }
        if y < self.y {
            return false;
        }
        if x > self.x + self.w {
            return false;
        }
        if y > self.y + self.h {
            return false;
        }
        true
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if other.x > self.x + self.w {
            return false;
        }
        if other.y > self.y + self.h {
            return false;
        }
        if other.x + other.w < self.x {
            return false;
        }
        if other.y + other.h < self.y {
            return false;
        }
        true
    }

    pub fn intersect(&self, other: Self) -> Option<Self> {
        if !self.overlaps(&other) {
            return None;
        }
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Some(Rect {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    }
}


struct Intersect<T:Float> {
    pos: Vec2<T>,
    t: T,
}

impl<T> Rect<T>
where T: Float + PartialOrd + MinMax + Copy + AddAssign + SubAssign + Default {

    fn line_intersection(line1: &Line<T>, line2: AxisLine<T>) -> Option<Intersect<T>> {
        match line2 {
            AxisLine::Vertical(x) => {
                let direction = line1.end.x - line1.start.x;
                let t = (x - line1.start.x) / direction;
                if t >= T::zero() && t <= T::one() {
                    let pos = Vec2 {
                        x, y: line1.start.y + t * (line1.end.y - line1.start.y),
                    };

                    Some(Intersect{ pos, t})
                } else {
                    None
                }
            }
            AxisLine::Horizontal(y) => {
                let direction = line1.end.y - line1.start.y;
                let t = (y - line1.start.y) / direction;
                if t >= T::zero() && t <= T::one() {
                    let pos = Vec2 {
                        x: line1.start.x + t * (line1.end.x - line1.start.x), y
                    };

                    Some(Intersect{ pos, t})
                } else {
                    None
                }
            }
        }
    }


    pub fn intersect_line(&self, line:&Line<T>) -> Option<IntermediateCollision<T>> {
        let direction_x = line.end.x - line.start.x;
        if direction_x > T::zero() {
            if let Some(i) = Self::line_intersection(line, AxisLine::Vertical(self.x)) {
                return Some(IntermediateCollision{ pos:i.pos, t:i.t, normal:Vec2::left() })
            }
        } else if let Some(i) = Self::line_intersection(line, AxisLine::Vertical(self.right())) { 
            return Some(IntermediateCollision{ pos:i.pos, t:i.t, normal:Vec2::right() })
        }

        let direction_y = line.end.y - line.start.y;
        if direction_y > T::zero() {
            if let Some(i) = Self::line_intersection(line, AxisLine::Horizontal(self.y)) {
                return Some(IntermediateCollision{ pos:i.pos, t:i.t, normal:Vec2::up() })
            }
        } else if let Some(i) = Self::line_intersection(line, AxisLine::Horizontal(self.bottom())) {
            return Some(IntermediateCollision{ pos:i.pos, t:i.t, normal:Vec2::down() })
        }
        None
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
        Rect {
            x: floorf(self.x) as i32,
            y: floorf(self.y) as i32,
            w: floorf(self.w) as i32,
            h: floorf(self.h) as i32,
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
