//! Simple pool allocation demo
#![allow(unused)]

use tato_arena::Arena;

#[derive(Debug, Default)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    println!("=== Pool Demo ===");

    let mut arena: Arena<1024> = Arena::new();

    // Create a pool of points
    let points = arena.alloc_pool::<Point>(5).unwrap();

    // Modify the points
    {
        let slice = arena.get_pool_mut(&points).unwrap();
        slice[0] = Point { x: 10, y: 20 };
        slice[1] = Point { x: 30, y: 40 };
        slice[2] = Point { x: 50, y: 60 };
    }

    // Read the points
    let slice = arena.get_pool(&points).unwrap();
    println!("Points: {:?}", slice);

    // Create another pool with initialization function
    let enemies = arena.alloc_pool_from_fn(3, |i| Point {
        x: i as i32 * 100,
        y: i as i32 * 50,
    }).unwrap();

    let enemy_slice = arena.get_pool(&enemies).unwrap();
    println!("Enemies: {:?}", enemy_slice);

    println!("Arena used: {} bytes", arena.used());
}
