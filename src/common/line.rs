use super::*;


pub struct Line<T> {
    pub start: Vec2<T>,
    pub end: Vec2<T>,
}


pub enum AxisLine<T> {
    Vertical(T),   // x-coordinate is constant
    Horizontal(T), // y-coordinate is constant
}