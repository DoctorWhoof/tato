#![allow(unused)]
//! Example showing how to use RawId for storing heterogeneous generic types

use tato_arena::{Arena, RawId};



// Example tilemap type with const generic size
#[derive(Debug)]
struct Tilemap<const LEN: usize> {
    tiles: [u8; LEN],
    width: u16,
    height: u16,
}

impl<const LEN: usize> Tilemap<LEN> {
    fn new(width: u16, height: u16, default_tile: u8) -> Self {
        assert_eq!((width as usize) * (height as usize), LEN);
        Self {
            tiles: [default_tile; LEN],
            width,
            height,
        }
    }

    fn set_tile(&mut self, x: u16, y: u16, tile: u8) {
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.tiles[idx] = tile;
    }

    fn get_tile(&self, x: u16, y: u16) -> u8 {
        let idx = (y as usize) * (self.width as usize) + (x as usize);
        self.tiles[idx]
    }
}

// Example: A game level with multiple tilemaps of different sizes
struct Level {
    // Store all tilemap IDs regardless of their size
    tilemap_ids: Vec<RawId>,
    // Keep track of what each tilemap is for
    tilemap_info: Vec<TilemapInfo>,
}

#[derive(Debug)]
enum TilemapInfo {
    Background { width: u16, height: u16 },
    Collision { width: u16, height: u16 },
    Decoration { width: u16, height: u16 },
}

impl Level {
    fn new() -> Self {
        Self {
            tilemap_ids: Vec::new(),
            tilemap_info: Vec::new(),
        }
    }

    fn add_tilemap<const LEN: usize>(
        &mut self,
        arena: &mut Arena<65536>,
        tilemap: Tilemap<LEN>,
        info: TilemapInfo,
    ) {
        if let Some(id) = arena.alloc(tilemap) {
            // Convert to RawId for storage
            self.tilemap_ids.push(id.raw());
            self.tilemap_info.push(info);
        }
    }

    // Get a specific tilemap when you know its size
    fn get_tilemap<'a, const LEN: usize>(
        &self,
        arena: &'a Arena<65536>,
        index: usize,
    ) -> Option<&'a Tilemap<LEN>> {
        let raw_id = self.tilemap_ids.get(index)?;
        let typed_id = raw_id.typed::<Tilemap<LEN>>();
        Some(arena.get(&typed_id))
    }
}

fn main() {
    // Create arena and level
    let mut arena: Arena<65536> = Arena::new();
    let mut level = Level::new();

    // Add different sized tilemaps
    // 16x16 background
    level.add_tilemap(
        &mut arena,
        Tilemap::<256>::new(16, 16, 0),
        TilemapInfo::Background { width: 16, height: 16 },
    );

    // 32x32 collision map
    level.add_tilemap(
        &mut arena,
        Tilemap::<1024>::new(32, 32, 0),
        TilemapInfo::Collision { width: 32, height: 32 },
    );

    // 8x8 decoration layer
    level.add_tilemap(
        &mut arena,
        Tilemap::<64>::new(8, 8, 0),
        TilemapInfo::Decoration { width: 8, height: 8 },
    );

    // Access tilemaps based on their info
    for (i, info) in level.tilemap_info.iter().enumerate() {
        match info {
            TilemapInfo::Background { width: 16, height: 16 } => {
                // We know this is a 16x16 = 256 tile map
                let tilemap = level.get_tilemap::<256>(&arena, i).unwrap();
                println!("Background tilemap: {}x{}", tilemap.width, tilemap.height);
            }
            TilemapInfo::Collision { width: 32, height: 32 } => {
                // We know this is a 32x32 = 1024 tile map
                let tilemap = level.get_tilemap::<1024>(&arena, i).unwrap();
                println!("Collision tilemap: {}x{}", tilemap.width, tilemap.height);
            }
            TilemapInfo::Decoration { width: 8, height: 8 } => {
                // We know this is a 8x8 = 64 tile map
                let tilemap = level.get_tilemap::<64>(&arena, i).unwrap();
                println!("Decoration tilemap: {}x{}", tilemap.width, tilemap.height);
            }
            _ => {}
        }
    }

    // Example of runtime type safety in debug mode
    #[cfg(debug_assertions)]
    {
        // This would panic in debug mode because we're trying to access
        // a 256-tile tilemap as if it were a 64-tile tilemap
        let _raw_id = level.tilemap_ids[0]; // This is the 256-tile background
        // let wrong_typed: ArenaId<Tilemap<64>> = raw_id.typed(); // Would panic!
    }

    println!("\nArena stats:");
    println!("  Used: {} bytes", arena.used());
    println!("  Remaining: {} bytes", arena.remaining());
    println!("  Allocations: {}", arena.allocation_count());
}
