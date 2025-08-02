//! Generic storage example using type markers

use tato_arena::Arena;

#[derive(Debug)]
struct Tilemap<const SIZE: usize> {
    width: u32,
    height: u32,
    tiles: [u8; SIZE],
}

impl<const SIZE: usize> Default for Tilemap<SIZE> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            tiles: [0; SIZE],
        }
    }
}

// Type markers for different storage types
struct GameStorage;
struct TempStorage;

fn main() {
    println!("=== Generic Storage Demo ===");

    // Create different arenas with type markers
    let mut game_arena: Arena<2048, usize, GameStorage> = Arena::new();
    let mut temp_arena: Arena<1024, usize, TempStorage> = Arena::new();

    // Store different sized tilemaps
    let small_map = game_arena.alloc(Tilemap::<256> {
        width: 16,
        height: 16,
        tiles: [1; 256],
    }).unwrap();

    let large_map = game_arena.alloc(Tilemap::<1024> {
        width: 32,
        height: 32,
        tiles: [2; 1024],
    }).unwrap();

    let temp_map = temp_arena.alloc(Tilemap::<128> {
        width: 8,
        height: 16,
        tiles: [3; 128],
    }).unwrap();

    // Access the maps
    if let Some(map) = game_arena.get(&small_map) {
        println!("Small map: {}x{}, first tile: {}", map.width, map.height, map.tiles[0]);
    }

    if let Some(map) = game_arena.get(&large_map) {
        println!("Large map: {}x{}, first tile: {}", map.width, map.height, map.tiles[0]);
    }

    if let Some(map) = temp_arena.get(&temp_map) {
        println!("Temp map: {}x{}, first tile: {}", map.width, map.height, map.tiles[0]);
    }

    // Type safety - these would not compile:
    // game_arena.get(&temp_map); // Error: type marker mismatch!
    // temp_arena.get(&small_map); // Error: type marker mismatch!

    println!("Game arena used: {} bytes", game_arena.used());
    println!("Temp arena used: {} bytes", temp_arena.used());

    // Convert to raw and back (useful for serialization)
    let raw_id = small_map.raw();
    println!("Raw ID generation: {}", raw_id.generation());

    let typed_back: tato_arena::ArenaId<Tilemap<256>, usize, GameStorage> = raw_id.typed();
    if let Some(map) = game_arena.get(&typed_back) {
        println!("Recovered map: {}x{}", map.width, map.height);
    }
}
