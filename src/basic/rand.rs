// use std::time::{SystemTime, UNIX_EPOCH};
use core::sync::atomic::{Ordering, AtomicI32};


static RAND_INC:AtomicI32 = AtomicI32::new(0);

/// Provide a seed for repeatable pseudo random number generation.
#[allow(unused)]
pub fn seed(value:i32){
    RAND_INC.store(value, Ordering::Relaxed);
}

/// Generates a seed from system clock, so every time you run your app you'll get a different sequence.
/// Warning: don't call within a loop or sometimes you'll get the same number more than once in a row.
/// Call it before entering the loop where "get()" or "get_in_range()"" is.
// #[allow(unused)]
// pub fn seed_from_clock(nanos:i32){
//     let nanos = SystemTime::now().duration_since(UNIX_EPOCH)
//         .unwrap_or_default()
//         .subsec_nanos() as i32;
//     seed(nanos);
// }


/// Get a random number between 0.0 and 1.0.
pub fn random() -> f32 {
    let inc = RAND_INC.load(Ordering::Acquire) as f64;
    let result = (inc.sin() * 1111111.1).fract().abs() as f32;
    RAND_INC.fetch_add(1, Ordering::Release);
    result
}

/// Get a random number in the provided range.
pub fn random_in_range(a:f32, b:f32) -> f32 {
    let range = b - a;
    (random() * range) + a
}