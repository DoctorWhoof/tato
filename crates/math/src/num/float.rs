use super::signed_num::SignedNum;

/// A Float trait that narrows Num to floating point types only.
/// Automatically implemented for f32 and f64.
pub trait Float: SignedNum {
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn round(self) -> Self;
    fn exp(self) -> Self;
    fn sqrt(self) -> Self;
    fn powi(self, exp: i32) -> Self;
    fn abs(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn epsilon() -> Self;
    fn pi() -> Self;
}
