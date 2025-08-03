//! Numeric traits and implementations for the math crate.

pub mod num;
pub mod signed_num;
pub mod integer;
pub mod float;
pub mod macros;

#[cfg(test)]
pub mod tests;

// Re-export the main traits
pub use num::Num;
pub use signed_num::SignedNum;
pub use integer::Integer;
pub use float::Float;
