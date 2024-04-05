use core:: ops::{Add, Sub, AddAssign, SubAssign};
use crate::Float;


/// A generic 2D vector.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2<T> {
    pub x:T,
    pub y:T
}

impl<T> Vec2<T>  {
    pub fn new(x:T, y:T) -> Self {
        Self {x, y}
    }
}


impl Vec2<i8> {
    pub fn to_f32(self) -> Vec2<f32> { Vec2 { x:self.x as f32, y: self.y as f32 } }
}


impl Vec2<f32> {
    // This seems sufficient to avoid seams between entities! Check the "test_pattern" example.
    // If you're seeing seams in neighbor entities, the problem is very likely in the logic positioning them,
    // specially if they "loop" around i.e. their coordinates reset every once in a while.
    pub fn to_i32(self) -> Vec2<i32> {
        Vec2 {
            x: self.x.floor() as i32,
            y: self.y.floor() as i32
        }
    }
}


impl<T> Vec2<T>
where T: Float {

    pub fn zero() -> Self {
        Self{ x:T::zero(), y:T::zero() }
    }


    pub fn one() -> Self {
        Self{ x:T::one(), y:T::one() }
    }


    pub fn up() -> Self {
        Self{ x:T::zero(), y:-T::one() }
    }


    pub fn down() -> Self {
        Self{ x:T::zero(), y:T::one() }
    }


    pub fn left() -> Self {
        Self{ x:-T::one(), y:T::zero() }
    }


    pub fn right() -> Self {
        Self{ x:T::one(), y:T::zero() }
    }


    pub fn horiz(&self) -> Self {
        Self{ x:self.x, y:T::zero() }
    }


    pub fn vert(&self) -> Self {
        Self{ x:T::zero(), y:self.y }
    }


    pub fn len(&self) -> T {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }


    pub fn is_longer_than_zero(&self) -> bool {
        self.x.abs() > T::zero() || self.y.abs() > T::zero()
    }


    pub fn floor(&self) -> Self {
        Self { x:self.x.floor(), y:self.y.floor() }
    }


    pub fn angle_between(&self, other:&Self) -> T {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let angle = dy.atan2(dx);
        if angle < T::zero() {
            angle + (T::one() + T::one()) * T::PI()
        } else {
            angle
        }
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


    pub fn dot(&self, other: &Self) -> T {
        (self.x * other.x) + (self.y * other.y)
    }

    
    pub fn scale(&self, factor: T) -> Self {
        Vec2{
            x: self.x * factor,
            y: self.y * factor
        }
    }


    pub fn reflect(v:Self, n:Self) -> Self {
        let two = T::one() + T::one();
        let dot_product = v.x * n.x + v.y * n.y;
        Vec2{
            x: v.x - two * dot_product * n.x,
            y: v.y - two * dot_product * n.y
        }    
    }


    pub fn rotate(&self, angle: T) -> Self {
        let cos_theta = angle.cos();
        let sin_theta = angle.sin();
        Vec2 {
            x: self.x * cos_theta - self.y * sin_theta,
            y: self.x * sin_theta + self.y * cos_theta,
        }
    }


    pub fn distance_to(&self, other: Self) -> T {
        let dist_x = other.x - self.x;
        let dist_y = other.y - self.y;
        ((dist_x * dist_x) + (dist_y * dist_y)).sqrt()
    }


}


impl<T> Add<Vec2<T>> for Vec2<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn add(self, other: Vec2<T>) -> Self::Output {
        Vec2{
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<T> Sub<Vec2<T>> for Vec2<T>
where T:Add<Output = T> + Sub<Output = T> + Copy + PartialOrd {
    type Output = Self;

    fn sub(self, other: Vec2<T>) -> Self::Output {
        Vec2{
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl<T> AddAssign<Vec2<T>> for Vec2<T>
where T:Add<Output = T> + AddAssign + Copy + PartialOrd {    
    fn add_assign(&mut self, other: Vec2<T>) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl<T> SubAssign<Vec2<T>> for Vec2<T>
where T:Sub<Output = T> + AddAssign + Copy + PartialOrd {    
    fn sub_assign(&mut self, other: Vec2<T>) {
        self.x += other.x;
        self.y += other.y;
    }
}



