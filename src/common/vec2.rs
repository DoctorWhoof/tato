use core::ops::{Add, Sub};
use libm::floorf;


/// A generic 2D vector.
#[derive(Clone, Copy, Debug)]
pub struct Vec2<T> {
    pub x:T,
    pub y:T
}


impl<T> Default for Vec2<T> where T:Default {
    fn default() -> Self {
        Self { x: Default::default(), y: Default::default() }
    }
}


impl Vec2<f32> {
    pub fn to_i32(self) -> Vec2<i32> { Vec2 { x: floorf(self.x) as i32, y: floorf(self.y) as i32 } }
}


impl<T> core::ops::Add<Vec2<T>> for Vec2<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn add(self, other: Vec2<T>) -> Self::Output {
        Vec2{
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> core::ops::Sub<Vec2<T>> for Vec2<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn sub(self, other: Vec2<T>) -> Self::Output {
        Vec2{
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

