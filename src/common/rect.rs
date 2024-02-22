use super::*;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
use core::f32::consts::{PI, FRAC_PI_2};
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
        if x >= self.x + self.w {
            return false;
        }
        if y >= self.y + self.h {
            return false;
        }
        true
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        if other.x >= self.x + self.w {
            return false;
        }
        if other.y >= self.y + self.h {
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

impl<T> Rect<T>
where
    T: Float + PartialOrd + MinMax + Copy + AddAssign + SubAssign + Default,
{
    // /// Adjusts the position of self along the x-axis to remove overlap with another rectangle
    // pub fn deintersect_x(&mut self, other: &Rect<T>) {
    //     // if doesn't overlap in x, skip it
    //     if other.y >= self.y + self.h || other.y + other.h <= self.y {
    //         return;
    //     }

    //     let self_right = self.right();
    //     let other_right = other.right();

    //     if self.x < other_right && self.x > other.x {
    //         let overlap = other_right - self.x;
    //         self.x += overlap;
    //     } else if self.x < other.x && self_right > other.x {
    //         let overlap = self_right - other.x;
    //         self.x -= overlap;
    //     }
    // }

    // /// Adjusts the position of self along the y-axis to remove overlap with another rectangle
    // pub fn deintersect_y(&mut self, other: &Rect<T>) {
    //     // if doesn't overlap in y, skip it
    //     if other.x >= self.x + self.w || other.x + other.w <= self.x {
    //         return;
    //     }

    //     let self_bottom = self.bottom();
    //     let other_bottom = other.bottom();

    //     if self.y < other_bottom && self.y > other.y {
    //         let overlap = other_bottom - self.y;
    //         self.y += overlap;
    //     } else if self.y < other.y && self_bottom > other.y {
    //         let overlap = self_bottom - other.y;
    //         self.y -= overlap;
    //     }
    // }

    // A ray starts at a point and extends into infinity
    // pub fn intersect_ray(&self, ray: &Ray<T>) -> Option<Vec2<T>> {
    //     let ray_direction = Vec2 {
    //         x: ray.angle.cos(),
    //         y: ray.angle.sin(),
    //     };

    //     let rectangle_sides = [
    //         (Vec2 { x: self.x, y: self.y }, Vec2 { x: self.x + self.w, y: self.y }), // Top
    //         (Vec2 { x: self.x, y: self.y }, Vec2 { x: self.x, y: self.y + self.h }), // Left
    //         (Vec2 { x: self.x, y: self.y + self.h }, Vec2 { x: self.x + self.w, y: self.y + self.h }), // Bottom
    //         (Vec2 { x: self.x + self.w, y: self.y }, Vec2 { x: self.x + self.w, y: self.y + self.h }), // Right
    //     ];

    //     for &(start, end) in &rectangle_sides {
    //         let edge_direction = Vec2 { x: end.x - start.x, y: end.y - start.y };
    //         let denom = ray_direction.x * edge_direction.y - ray_direction.y * edge_direction.x;

    //         if denom.abs() > T::epsilon() {
    //             let d = Vec2 { x: start.x - ray.origin.x, y: start.y - ray.origin.y };
    //             let numerator = d.x * edge_direction.y - d.y * edge_direction.x;
    //             let t = numerator / denom;

    //             if t >= T::zero() {
    //                 let u = (d.x * ray_direction.y - d.y * ray_direction.x) / denom;
    //                 if u >= T::zero() && u <= T::one() {
    //                     return Some(Vec2 {
    //                         x: ray.origin.x + t * ray_direction.x,
    //                         y: ray.origin.y + t * ray_direction.y,
    //                     });
    //                 }
    //             }
    //         }
    //     }

    //     None
    // }


    pub fn intersect_ray(&self, ray: &Ray<T>) -> Option<(Vec2<T>, f32)> {
        let ray_direction = Vec2 {
            x: ray.angle.cos(),
            y: ray.angle.sin(),
        };

        let rectangle_sides = [
            (Vec2 { x: self.x, y: self.y }, Vec2 { x: self.x + self.w, y: self.y }, FRAC_PI_2), // Top
            (Vec2 { x: self.x, y: self.y }, Vec2 { x: self.x, y: self.y + self.h }, PI), // Left
            (Vec2 { x: self.x, y: self.y + self.h }, Vec2 { x: self.x + self.w, y: self.y + self.h }, -FRAC_PI_2), // Bottom
            (Vec2 { x: self.x + self.w, y: self.y }, Vec2 { x: self.x + self.w, y: self.y + self.h }, 0.0), // Right
        ];

        for &(start, end, normal_angle) in &rectangle_sides {
            let edge_direction = Vec2 { x: end.x - start.x, y: end.y - start.y };
            let denom = ray_direction.x * edge_direction.y - ray_direction.y * edge_direction.x;

            if denom.abs() > T::epsilon() {
                let d = Vec2 { x: start.x - ray.origin.x, y: start.y - ray.origin.y };
                let numerator = d.x * edge_direction.y - d.y * edge_direction.x;
                let t = numerator / denom;

                if t >= T::zero() {
                    let u = (d.x * ray_direction.y - d.y * ray_direction.x) / denom;
                    if u >= T::zero() && u <= T::one() {
                        return Some((Vec2 {
                            x: ray.origin.x + t * ray_direction.x,
                            y: ray.origin.y + t * ray_direction.y,
                        }, normal_angle));
                    }
                }
            }
        }

        None
    }

    /// Returns the collision point and normal (in radians)
    pub fn intersect_line(&self, line: Line<T>) -> Option<Collision<T>> {
        let mut closest_intersection: Option<Collision<T>> = None;

        self.update_intersection_point(
            &line,
            &Line {
                start: Vec2 {
                    y: self.bottom(),
                    x: self.x,
                },
                end: Vec2 {
                    x: self.right(),
                    y: self.bottom(),
                },
            },
            &mut closest_intersection,
            270.0 * DEG_TO_RAD, // Bottom
        );

        self.update_intersection_point(
            &line,
            &Line {
                start: Vec2 {
                    x: self.x,
                    y: self.y,
                },
                end: Vec2 {
                    x: self.right(),
                    y: self.y,
                },
            },
            &mut closest_intersection,
            90.0 * DEG_TO_RAD, // Top
        );

        self.update_intersection_point(
            &line,
            &Line {
                start: Vec2 {
                    x: self.x,
                    y: self.y,
                },
                end: Vec2 {
                    x: self.x,
                    y: self.bottom(),
                },
            },
            &mut closest_intersection,
            180.0 * DEG_TO_RAD, // Left
        );

        self.update_intersection_point(
            &line,
            &Line {
                start: Vec2 {
                    x: self.right(),
                    y: self.y,
                },
                end: Vec2 {
                    x: self.right(),
                    y: self.bottom(),
                },
            },
            &mut closest_intersection,
            0.0, // Right
        );

        closest_intersection
    }

    fn update_intersection_point(
        &self,
        line1: &Line<T>,
        line2: &Line<T>,
        closest: &mut Option<Collision<T>>,
        normal: f32,
    ) {
        let x = line1.start.x;
        let y = line1.start.y;
        let x2 = line1.end.x;
        let y2 = line1.end.y;
        let x3 = line2.start.x;
        let y3 = line2.start.y;
        let x4 = line2.end.x;
        let y4 = line2.end.y;

        let denominator = (x - x2) * (y3 - y4) - (y - y2) * (x3 - x4);

        if !denominator.is_zero() {
            let t = ((x - x3) * (y3 - y4) - (y - y3) * (x3 - x4)) / denominator;
            let u = -((x - x2) * (y - y3) - (y - y2) * (x - x3)) / denominator;

            if t >= T::zero() && t <= T::one() && u >= T::zero() && u <= T::one() {
                let intersection_x = x + t * (x2 - x);
                let intersection_y = y + t * (y2 - y);

                match closest {
                    Some(closest_point) => {
                        if t < closest_point.point.x {
                            *closest = Some(Collision {
                                point: Vec2 {
                                    x: intersection_x,
                                    y: intersection_y,    
                                },
                                normal,
                                .. Default::default()
                            });
                        }
                    }
                    None => {
                        *closest = Some(Collision {
                            point: Vec2 {
                                x: intersection_x,
                                y: intersection_y,
                            },
                            normal,
                            .. Default::default()
                        });
                    }
                }
            }
        }
    }


    fn overlap_amount(&self, other: &Rect<T>) -> (T, T) {
        let dx = Float::min(self.x + self.w - other.x, other.x + other.w - self.x);
        let dy = Float::min(self.y + self.h - other.y, other.y + other.h - self.y);
        (Float::max(dx, T::zero()), Float::max(dy, T::zero())) // Ensure no negative overlap
    }



    pub fn deintersect(&mut self, vel:Vec2<T>, other:&Rect<T>){
        let dx = if vel.x > T::zero() {
            other.x - (self.x + self.w)
        } else {
            self.x - (other.x + other.w)
        };

        let dy = if vel.y > T::zero() {
            other.y - (self.y + self.h)
        } else {
            self.y - (other.y + other.h)
        };
    
        self.x += dx;
        self.y += dy;
        // let dx = if self.x < other.x {
        //     other.x - (self.x + self.w)
        // } else {
        //     self.x - (other.x + other.w)
        // };

        // let dy = if self.y < other.y {
        //     other.y - (self.y + self.h)
        // } else {
        //     self.y - (other.y + other.h)
        // };

        // // // Normalize the velocity vector to get its direction.
        // let velocity_direction = vel.normalize();
    
        // // // Scale the mtv by the normalized velocity direction of r1.
        // // // This ensures that the adjustment respects the direction of r1's velocity.
        // let adjustment = Vec2 {
        //     x: if velocity_direction.x != T::zero(){
        //         dx.abs() * -velocity_direction.x
        //     } else {
        //         T::zero()
        //     },
        //     y: if velocity_direction.y != T::zero() {
        //         dy.abs() * -velocity_direction.y
        //     } else {
        //         T::zero()
        //     },
        // };
    
        // self.x += adjustment.x;
        // self.y += adjustment.y;
    }


    pub fn deintersect_both(
        r1: &mut Rect<T>,
        r1_vel:Vec2<T>,
        r1_factor:T,
        r2: &mut Rect<T>,
        r2_vel:Vec2<T>,
        r2_factor:T,
    ){
        let (overlap_x, overlap_y) = r1.overlap_amount(r2);
    
        // Avoid adjusting if there's no overlap
        if overlap_x == T::zero() && overlap_y == T::zero() {
            return;
        }
    
        // Calculate velocity influence as the magnitude of each rectangle's velocity
        let vel_influence_r1 = r1_vel.len() * r1_factor;
        let vel_influence_r2 = r2_vel.len() * r2_factor;
        let total_vel_influence = vel_influence_r1 + vel_influence_r2;
    
        // Ensure there's an influence factor to work with
        if total_vel_influence == T::zero() {
            return;
        }
    
        // Calculate the proportion of the overlap each rect should cover based on their velocity influence
        let r1_move_proportion = vel_influence_r1 / total_vel_influence;
        let r1_move_x = overlap_x * r1_move_proportion;
        let r1_move_y = overlap_y * r1_move_proportion;
    
        // Adjust positions based on calculated proportions and direction of velocities
        let r1_adjust_x = if r1_vel.x != T::zero() { -r1_move_x } else { r1_move_x };
        let r1_adjust_y = if r1_vel.y != T::zero() { -r1_move_y } else { r1_move_y };
        let r2_adjust_x = overlap_x - r1_move_x;
        let r2_adjust_y = overlap_y - r1_move_y;
    
        r1.x += r1_adjust_x;
        r1.y += r1_adjust_y;
        r2.x += r2_adjust_x;
        r2.y += r2_adjust_y;
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
