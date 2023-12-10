use super::*;
use libm::floorf;
use core::ops::{Add, Sub, Mul};

#[derive(Clone, Copy, Debug)]
pub struct Rect<T> {
    pub x:T,
    pub y:T,
    pub w:T,
    pub h:T
}


impl<T> Default for Rect<T>
where T:Default {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default(), w: Default::default(), h: Default::default() }
    }
}


impl<T> Rect<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd + MinMax {
    pub fn new(x:T, y:T, w:T, h:T) -> Self {
        Rect { x, y , w, h }
    }


    pub fn pos(&self) -> Vec2<T> {
        Vec2{ x:self.x, y:self.y }
    }


    // pub fn size(&self) -> Vec2<T> {
    //     Vec2{ x:self.w, y:self.h }
    // }


    pub fn contains(&self, x:T, y:T) -> bool {
        if x < self.x { return false }
        if y < self.y {return false }
        if x >= self.x + self.w { return false }
        if y >= self.y + self.h { return false }
        true
    }


    pub fn overlaps(&self, other:&Self) -> bool {
        if other.x >= self.x + self.w { return false }
        if other.y >= self.y + self.h { return false }
        if other.x + other.w < self.x { return false }
        if other.y + other.h < self.y { return false }
        true
    }


     pub fn intersect(&self, other:Self) -> Option<Self> {
        if !self.overlaps(&other) { return None }
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        Some(Rect { x, y, w: right-x, h: bottom-y })
    }


    // pub fn intersect(&self, other:&Self) -> Self {
    //     Rect {
    //         x: T::min(self.x, other.x),
    //         y: todo!(),
    //         w: todo!(),
    //         h: todo!(),
    //     }
    // }
    

    pub fn right(&self) -> T { self.x + self.w }
    
    
    pub fn bottom(&self) -> T { self.y + self.h }
    
}


impl From<Rect<u8>> for Rect<i32> {
    fn from(val: Rect<u8>) -> Self { Rect { x:val.x.into(), y:val.y.into(), w:val.w.into(), h:val.h.into() } }
}


impl From<Rect<i8>> for Rect<i32> {
    fn from(val: Rect<i8>) -> Self { Rect { x:val.x.into(), y:val.y.into(), w:val.w.into(), h:val.h.into() } }
}


impl Rect<f32> {
    pub fn to_i32(self) -> Rect<i32> {
        Rect{ x: floorf(self.x) as i32, y:floorf(self.y) as i32, w:floorf(self.w) as i32, h:floorf(self.h) as i32 }
    }
}

// Add/Sub a a rect's position to another rect's position
impl<T> Add<Rect<T>> for Rect<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn add(self, other: Rect<T>) -> Self::Output {
        Rect{
            x: self.x + other.x,
            y: self.y + other.y,
            w: self.w,
            h: self.h,
        }
    }
}


impl<T> Sub<Rect<T>> for Rect<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn sub(self, other: Rect<T>) -> Self::Output {
        Rect{
            x: self.x - other.x,
            y: self.y - other.y,
            w: self.w,
            h: self.h,
        }
    }
}

// Add/Sub a position vector from rect's position
impl<T,V> Add<Vec2<V>> for Rect<T>
where
T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
V:Into<T> {
    type Output = Self;

    fn add(self, other: Vec2<V>) -> Self::Output {
        Rect{
            x: self.x + other.x.into(),
            y: self.y + other.y.into(),
            w: self.w,
            h: self.h,
        }
    }
}

impl<T,V> Sub<Vec2<V>> for Rect<T>
where
T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd,
V:Into<T> {
    type Output = Self;

    fn sub(self, other: Vec2<V>) -> Self::Output {
        Rect{
            x: self.x - other.x.into(),
            y: self.y - other.y.into(),
            w: self.w,
            h: self.h,
        }
    }
}

// Multiply by T
impl<T> Mul<T> for Rect<T>
where T:Mul<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn mul(self, other:T) -> Self::Output {
        Rect{
            x: self.x * other,
            y: self.y * other,
            w: self.w,
            h: self.h,
        }
    }
}
