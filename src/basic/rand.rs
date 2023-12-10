use libm::{sin, fabs};
use core::sync::atomic::{Ordering, AtomicI32};

static RAND_INC:AtomicI32 = AtomicI32::new(0);

/// Provide a seed for repeatable pseudo random number generation.
#[allow(unused)]
pub fn seed(value:i32){
    RAND_INC.store(value, Ordering::Relaxed);
}

/// Get a random number between 0.0 and 1.0.
pub fn random() -> f32 {
    let inc = RAND_INC.load(Ordering::Acquire) as f64;
    let result = fract(fabs(sin(inc) * 1111111.1)) as f32;
    RAND_INC.fetch_add(1, Ordering::Release);
    result
}

/// Get a random number in the provided range.
pub fn random_in_range(a:f32, b:f32) -> f32 {
    let range = b - a;
    (random() * range) + a
}

fn fract(value:f64) -> f64 {
    value - ((value as i64) as f64)
}