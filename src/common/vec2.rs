use core::{f32::consts::PI, ops::{Add, Sub}};
use libm::floorf;
use num_traits::Float;


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
    }


    pub fn is_longer_than_zero(&self) -> bool {
        self.x.abs() > T::zero() || self.y.abs() > T::zero()
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

    pub fn dot(&self, other: &Self) -> T {
        (self.x * other.x) + (self.y * other.y)
    }

    pub fn scale(&self, factor: T) -> Self {
        Vec2{
            x: self.x * factor,
            y: self.y * factor
        }
    }

    // TODO: implement as Sub trait
    pub fn subtract(&self, other: &Self) -> Self {
        Vec2{
            x:self.x - other.x,
            y:self.y - other.y
        }
    }

    pub fn reflect(velocity: Self, collision_normal: T) -> Self {
        let two = T::one() + T::one();
        let normal = Vec2{
            x:collision_normal.cos(),
            y:collision_normal.sin()
        };
        let dot_product = velocity.dot(&normal);
        velocity.subtract(&normal.scale(two * dot_product))
    }

    pub fn weighted_add(v1:Self, v2:Self, weight_1: T, weight_2: T) -> Self {
        Self{
            x: ((v1.x * weight_1) + (v2.x * weight_2)),
            y: ((v1.y * weight_1) + (v2.y * weight_2)),
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

    // pub fn blend(v1:Self, v2:Self, weight_1: T, weight_2: T, invert_y:bool) -> Self {
    //     let two = T::one() + T::one();
    //     if invert_y {
    //         Self{
    //             x: ((v1.x * weight_1) + (v2.x * weight_2)) / two,
    //             y: ((v1.y * weight_1) + (-v2.y * weight_2)) / two,
    //         }
    //     } else {
    //         Self{
    //             x: ((v1.x * weight_1) + (v2.x * weight_2)) / two,
    //             y: ((v1.y * weight_1) +( v2.y * weight_2)) / two,
    //         }
    //     }
    // }

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



