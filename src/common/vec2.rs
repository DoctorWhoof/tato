use core::{f32::consts::PI, ops::{Add, Sub}};
use libm::floorf;
use num_traits::Float;


/// A generic 2D vector.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2<T> {
    pub x:T,
    pub y:T
}


impl Vec2<i8> {

    pub fn to_f32(self) -> Vec2<f32> { Vec2 { x:self.x as f32, y: self.y as f32 } }

}


impl Vec2<f32> {

    pub fn to_i32(self) -> Vec2<i32> { Vec2 { x: floorf(self.x) as i32, y: floorf(self.y) as i32 } }

    // Can't figure out a way to make PI generic without complicating the API
    pub fn angle_between(&self, other:&Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let angle = dy.atan2(dx);
        if angle < 0.0 {
            angle + 2.0 * PI
        } else {
            angle
        }
    }
    
}


impl<T:Float> Vec2<T> {

    pub fn zero() -> Self {
        Self{ x:T::zero(), y:T::zero() }
    }

    pub fn len(&self) -> T {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
        // ((self.x * self.x) + (self.y * self.y)).sqrt().abs()
    }


    pub fn floor(&self) -> Self {
        Self { x:self.x.floor(), y:self.y.floor() }
    }


    pub fn normalize(&self) -> Self {
        let len = self.len();
        if len > T::epsilon() {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            *self
        }
    }

    
    pub fn clamp_to_length(self, max_length: T) -> Vec2<T> {
        let current_length = self.len();
        if current_length > max_length && current_length > T::epsilon() {
            let normalized = self.normalize();
            Vec2 {
                x: normalized.x * max_length,
                y: normalized.y * max_length,
            }
        } else {
            self
        }
    }

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



