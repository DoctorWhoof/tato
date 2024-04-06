
pub trait Float:
    num_traits::Float +
    num_traits::FloatConst +
    num_traits::FromPrimitive +
    num_traits::ToPrimitive +
    core::ops::AddAssign +
    core::ops::SubAssign +
    core::ops::MulAssign +
    core::ops::DivAssign +
    core::fmt::Debug +
    core::fmt::Display + 
{
    fn two() -> Self;
}

impl Float for f32 {
    fn two() -> f32 { 2.0 }
}

impl Float for f64 {
    fn two() -> f64 { 2.0 }
}