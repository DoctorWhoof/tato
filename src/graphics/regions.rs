// use tato_video::Tile;
// use crate::prelude::TILES_DEFAULT;

// /// Macro to extract a range of tiles as a const array
// /// Usage: tile_region!(source, start..end)
// macro_rules! tile_region {
//     ($source:expr, $start:literal..$end:literal) => {{
//         const LEN: usize = $end - $start;
//         const START: usize = $start;

//         const fn extract() -> [Tile<2>; LEN] {
//             let mut result = [Tile::<2>::new(0, 0); LEN];
//             let mut i = 0;
//             while i < LEN {
//                 result[i] = $source.tiles[START + i];
//                 i += 1;
//             }
//             result
//         }

//         extract()
//     }};
// }

// pub const DITHER: [Tile<2>; 9] = tile_region!(TILES_DEFAULT, 0..9);
// pub const LINES: [Tile<2>; 7] = tile_region!(TILES_DEFAULT, 9..16);
// pub const NUMBERS: [Tile<2>; 10] = tile_region!(TILES_DEFAULT, 16..26);
// pub const CHARS: [Tile<2>; 74] = tile_region!(TILES_DEFAULT, 26..100);
// pub const ICONS: [Tile<2>; 11] = tile_region!(TILES_DEFAULT, 100..111);
// pub const DIAGONALS: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 112..128);
// pub const SURFACES: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 128..144);
// pub const SYMBOLS: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 144..160);
// pub const FRAMES: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 160..176);
// pub const CIRCLES: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 176..192);
// pub const GRIDS: [Tile<2>; 16] = tile_region!(TILES_DEFAULT, 192..208);
// From here on to 255: may need redesign, saving for later
