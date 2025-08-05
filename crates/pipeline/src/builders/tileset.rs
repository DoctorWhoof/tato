use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_LEN];
pub(crate) type CanonicalTile = [u8; TILE_LEN]; // Colors remapped to canonical form (0,1,2,3...) to allow detection of palette-swapped tiles!

pub struct TilesetBuilder<'a> {
    pub allow_tile_transforms: bool,
    pub allow_unused: bool,
    pub use_crate_assets: bool,
    pub save_colors: bool,
    pub name: String,
    pub pixels: Vec<u8>,
    pub tile_hash: HashMap<CanonicalTile, Cell>,
    pub sub_palette_name_hash: HashMap<[u8; COLORS_PER_TILE as usize], String>,
    pub sub_palettes: Vec<[u8; COLORS_PER_TILE as usize]>,
    groups: GroupBuilder,
    anims: Vec<AnimBuilder>,
    maps: Vec<MapBuilder>,
    single_tiles: Vec<SingleTileBuilder>,
    palette: &'a mut PaletteBuilder,
    next_tile: u8,
    sub_palette_head: usize,
}

impl<'a> TilesetBuilder<'a> {
    pub fn new(name: &str, palette: &'a mut PaletteBuilder) -> Self {
        Self {
            allow_tile_transforms: true,
            allow_unused: false,
            use_crate_assets: false,
            save_colors: true,
            name: String::from(name),
            pixels: vec![],
            tile_hash: HashMap::new(),
            groups: GroupBuilder::default(),
            sub_palette_name_hash: HashMap::new(),
            next_tile: 0,
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            palette,
            sub_palettes: Vec::new(),
            sub_palette_head: 0,
        }
    }

    fn load_valid_image(&mut self, path: &str, frames_h: u8, frames_v: u8) -> PalettizedImg {
        let img = PalettizedImg::from_image(path, frames_h, frames_v, self.palette);
        assert!(
            img.width % TILE_SIZE as usize == 0,
            "Single tile width must be multiple of {}",
            TILE_SIZE
        );
        img
    }

    /// Defines a new tile group. Adds the tiles only, does not add Tilemaps or Animations,
    /// those must be added afterwards and will be correctly marked as part of a group, if
    /// there's a match.
    pub fn new_group(&mut self, path: &str, name: &str) {
        let group_index = self.groups.names.len() + 1;
        assert!(group_index > 0 && group_index <= 16, "Group index must be between 1-16");
        let group_index = group_index as u8;

        let img = self.load_valid_image(path, 1, 1);
        // Ensure the names vec is large enough and store the group name
        let vec_index = (group_index - 1) as usize; // Convert 1-based to 0-based
        if self.groups.names.len() <= vec_index {
            self.groups.names.resize(vec_index + 1, String::new());
        }
        self.groups.names[vec_index] = String::from(name);

        // Process tiles and register them in the group, discard the returned cells
        let _ = self.add_tiles(&img, Some(group_index));
    }


    /// Creates a new single tile from a .png file
    pub fn new_tile(&mut self, path: &str) {
        let img = self.load_valid_image(path, 1, 1);
        // Additional asserts for single tile
        assert!(img.width == TILE_SIZE as usize, "Single tile width must be {}", TILE_SIZE);
        assert!(
            img.cols_per_frame == 1 && img.rows_per_frame == 1,
            "Single tile must be 1x1 tile (8x8 pixels)"
        );

        let cells = self.add_tiles(&img, None);
        assert!(
            cells.len() == 1 && cells[0].len() == 1,
            "Single tile should produce exactly one cell"
        );

        let tile_name = crate::strip_path_name(path);
        self.single_tiles.push(SingleTileBuilder { name: tile_name, cell: cells[0][0] });
    }

    /// Creates a new map from a .png file
    pub fn new_map(&mut self, path: &str, name: &str) {
        let img = self.load_valid_image(path, 1, 1);
        let frames = self.add_tiles(&img, None);
        assert!(frames.len() == 1);

        let map = MapBuilder {
            name: String::from(name),
            columns: u8::try_from(img.cols_per_frame).unwrap(),
            rows: u8::try_from(img.rows_per_frame).unwrap(),
            cells: frames[0].clone(), // just the first frame, there's only 1 anyway!
        };

        self.maps.push(map);
    }

    /// Creates a new animation strip from a .png file
    pub fn new_animation_strip(&mut self, path: &str, name: &str, frames_h: u8, frames_v: u8) {
        let img = self.load_valid_image(path, frames_h, frames_v);
        let cells = self.add_tiles(&img, None);
        let frame_count = img.frames_h as usize * img.frames_v as usize;

        assert!(frame_count > 0);
        let anim = AnimBuilder {
            name: String::from(name),
            frames: (0..frame_count)
                .map(|i| MapBuilder {
                    name: format!("frame_{:02}", i),
                    columns: u8::try_from(img.cols_per_frame).unwrap(),
                    rows: u8::try_from(img.rows_per_frame).unwrap(),
                    cells: cells[i].clone(),
                })
                .collect(),
            // tags: vec![],
        };

        self.anims.push(anim);
    }

    /// Writes the tileset constants to a file
    pub fn write(&self, file_path: &str) {
        let mut code = CodeWriter::new(file_path);

        // Write header
        code.write_header(self.allow_unused, self.use_crate_assets);

        // Write tileset data structure
        code.write_tileset_data_struct(&self.name, self.save_colors, self.sub_palettes.len());

        // Write palette colors
        if self.save_colors {
            code.write_color_array(&self.name, &self.palette.colors);
        }

        // Write sub-palettes
        if self.save_colors {
            for (i, sub_palette) in self.sub_palettes.iter().enumerate() {
                code.write_sub_palette(&self.name, i, sub_palette);
            }
        }

        // Write group constants
        if !self.groups.names.is_empty() {
            for (index, name) in self.groups.names.iter().enumerate() {
                if !name.is_empty() {
                    let group_index = (index + 1) as u8; // Convert 0-based back to 1-based
                    code.write_group_constant(name, group_index);
                }
            }
            code.write_line("");
        }

        // Write animation strips if any
        if !self.anims.is_empty() {
            for anim in &self.anims {
                code.write_animation_strip(&anim.name, &anim.frames);
            }
        }

        // Write maps if any
        for map in &self.maps {
            code.write_tilemap_constant(&map.name, map.columns, map.rows, &map.cells);
        }

        // Write single tiles
        for tile in &self.single_tiles {
            if self.name == "default" {
                // For default tileset, generate TileID constants for type safety
                code.write_tile_id_constant(&tile.name, tile.cell.id.0);
            } else {
                code.write_cell_constant(&tile.name, tile.cell);
            }
        }
        if !self.single_tiles.is_empty() {
            code.write_line("");
        }

        // Write tile pixel data
        if !self.pixels.is_empty() {
            let tiles_count = self.pixels.len() / (TILE_SIZE as usize * TILE_SIZE as usize);
            code.write_tile_array_header(&self.name, tiles_count);

            for i in 0..tiles_count {
                let start = i * (TILE_SIZE as usize * TILE_SIZE as usize);
                let end = start + (TILE_SIZE as usize * TILE_SIZE as usize);
                let tile_pixels = &self.pixels[start..end];

                code.write_line("    Tile {");
                code.write_line("        clusters: [");

                // Generate 8 clusters (one per row), each with 8 pixels packed into 2 bytes
                for row in 0..8 {
                    let row_start = row * 8;
                    let row_end = row_start + 8;
                    let row_pixels = &tile_pixels[row_start..row_end];

                    // Pack 8 pixels (2 bits each) into 2 bytes
                    // 4 pixels per byte: pixels 0-3 in byte0, pixels 4-7 in byte1
                    let mut byte0 = 0u8;
                    let mut byte1 = 0u8;

                    for (i, &pixel) in row_pixels.iter().enumerate() {
                        let pixel = pixel & 0x3; // Ensure pixel fits in 2 bits
                        if i < 4 {
                            // Pack into first byte (pixels 0-3)
                            byte0 |= pixel << (6 - (i * 2));
                        } else {
                            // Pack into second byte (pixels 4-7)
                            byte1 |= pixel << (6 - ((i - 4) * 2));
                        }
                    }

                    code.write_tile_cluster(byte0, byte1);
                }

                code.write_line("        ],");
                code.write_line("    },");
            }
            code.write_line("];");
        }

        // Format the output
        code.format_output(file_path);
    }

    fn add_tiles(&mut self, img: &PalettizedImg, group:Option<u8>) -> Vec<Vec<Cell>> {
        let mut frames = vec![];

        // Main detection routine.
        // Iterate animation frames, then tiles within frames.
        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                let mut frame_tiles = vec![];
                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        // Extract tile pixels
                        let mut tile_data = [0u8; TILE_LEN];
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        for y in 0..TILE_SIZE as usize {
                            for x in 0..TILE_SIZE as usize {
                                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                                let index = (img.width * abs_y) + abs_x;
                                tile_data[(TILE_SIZE as usize * y) + x] = img.pixels[index];
                            }
                        }

                        // Create canonical representation
                        let (canonical_tile, color_mapping) = create_canonical_tile(&tile_data);

                        // If we're registering a group, store this canonical pattern (but skip empty tiles)
                        if let Some(group_idx) = group {
                            // Only register multi-color tiles in groups (skip empty/single-color tiles)
                            if color_mapping.len() > 1 {
                                let group_bit = 1u8 << (group_idx - 1); // Convert 1-based index to bit position
                                let current_groups = self.groups.hash.get(&canonical_tile).unwrap_or(&0);
                                self.groups.hash.insert(canonical_tile, current_groups | group_bit);

                                // Also register all transformations if enabled
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue; // Skip identity transform
                                                }

                                                let transformed_tile = transform_tile(&tile_data, flip_x, flip_y, rotation);
                                                let (transformed_canonical, transformed_colors) = create_canonical_tile(&transformed_tile);

                                                // Only register if the transformed tile is also multi-color
                                                if transformed_colors.len() > 1 {
                                                    let current_groups = self.groups.hash.get(&transformed_canonical).unwrap_or(&0);
                                                    self.groups.hash.insert(transformed_canonical, current_groups | group_bit);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }


                        if color_mapping.len() > SUBPALETTE_COUNT as usize {
                            panic!(
                                "\x1b[31mVideochip Error: \x1b[33mTile exceeds {} color limit!\n\
                                \tFrame: ({}, {})\n\
                                \tTile within frame: row {}, col {}\n\
                                \tAbsolute tile position: row {}, col {}\n\
                                \tFound {} unique colors\x1b[0m",
                                SUBPALETTE_COUNT,
                                frame_h,
                                frame_v,
                                row,
                                col,
                                abs_row,
                                abs_col,
                                color_mapping.len()
                            );
                        }

                        // Handle single-color tiles efficiently
                        let (sub_palette_id, remapping) = if color_mapping.len() <= 1 {
                            // Single color tile - find or create a simple sub-palette
                            self.find_or_create_single_color_sub_palette(color_mapping.get(0).copied().unwrap_or(0))
                        } else {
                            // Multi-color tile - use normal processing
                            self.find_or_create_compatible_sub_palette(&color_mapping)
                        };

                        // Check if this canonical tile (or any transformation) exists
                        let mut found_cell = None;
                        let mut normalized_tile = [0u8; TILE_LEN];
                        for (i, &canonical_index) in canonical_tile.iter().enumerate() {
                            normalized_tile[i] = remapping[canonical_index as usize];
                        }

                        // Check original first using remapped data
                        if let Some(existing) = self.tile_hash.get(&normalized_tile) {
                            found_cell = Some(*existing);
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations using remapped data
                            'outer: for flip_x in [false, true] {
                                for flip_y in [false, true] {
                                    for rotation in [false, true] {
                                        if !flip_x && !flip_y && !rotation {
                                            continue;
                                        }

                                        let transformed_original =
                                            transform_tile(&tile_data, flip_x, flip_y, rotation);
                                        let (transformed_canonical, transformed_colors) =
                                            create_canonical_tile(&transformed_original);

                                        // Apply remapping to transformed data
                                        let mut transformed_normalized = [0u8; TILE_LEN];
                                        for (i, &canonical_index) in
                                            transformed_canonical.iter().enumerate()
                                        {
                                            if (canonical_index as usize) < transformed_colors.len()
                                            {
                                                let color =
                                                    transformed_colors[canonical_index as usize];
                                                let original_index = color_mapping
                                                    .iter()
                                                    .position(|&c| c == color)
                                                    .unwrap_or(0);
                                                transformed_normalized[i] =
                                                    remapping[original_index];
                                            } else {
                                                transformed_normalized[i] = 0;
                                            }
                                        }

                                        if let Some(existing) =
                                            self.tile_hash.get(&transformed_normalized)
                                        {
                                            found_cell = Some(*existing);
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        // Look up group membership for this tile pattern
                        let group_bits = self.groups.hash.get(&canonical_tile).copied().unwrap_or(0);

                        let cell = match found_cell {
                            Some(existing_cell) => {
                                // Found existing tile with same pattern
                                // Use the same sub-palette mapping we used for lookup
                                Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id)
                                }
                            },
                            None => {
                                // Create new tile using the sub-palette we already found/created
                                let new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id)
                                };

                                // Store the already computed normalized_tile tile data
                                self.pixels.extend_from_slice(&normalized_tile);

                                // Store remapped tile in hash (after remapping is complete)
                                self.tile_hash.insert(normalized_tile, new_tile);

                                // Store all transformations using remapped data
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue;
                                                }

                                                let transformed_original = transform_tile(
                                                    &tile_data, flip_x, flip_y, rotation,
                                                );
                                                let (transformed_canonical, transformed_colors) =
                                                    create_canonical_tile(&transformed_original);

                                                // Apply same remapping to transformed data
                                                let mut transformed_normalized = [0u8; TILE_LEN];
                                                for (i, &canonical_index) in
                                                    transformed_canonical.iter().enumerate()
                                                {
                                                    if (canonical_index as usize)
                                                        < transformed_colors.len()
                                                    {
                                                        // Find this color in our original color mapping
                                                        let color = transformed_colors
                                                            [canonical_index as usize];
                                                        let original_index = color_mapping
                                                            .iter()
                                                            .position(|&c| c == color)
                                                            .unwrap_or(0);
                                                        transformed_normalized[i] =
                                                            remapping[original_index];
                                                    } else {
                                                        transformed_normalized[i] = 0;
                                                    }
                                                }

                                                // Only store if this transformation produces different data
                                                if !self
                                                    .tile_hash
                                                    .contains_key(&transformed_normalized)
                                                {
                                                    let mut cell_with_flags = new_tile;
                                                    cell_with_flags.flags.set_flip_x(flip_x);
                                                    cell_with_flags.flags.set_flip_y(flip_y);
                                                    cell_with_flags.flags.set_rotation(rotation);

                                                    self.tile_hash.insert(
                                                        transformed_normalized,
                                                        cell_with_flags,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                self.next_tile += 1;

                                new_tile
                            },
                        };

                        frame_tiles.push(cell);
                    }
                }
                frames.push(frame_tiles);
            }
        }

        frames
    }

    fn find_or_create_compatible_sub_palette(&mut self, colors: &[u8]) -> (u8, Vec<u8>) {
        // Work with unique colors only to avoid issues with repeated colors
        let unique_colors: Vec<u8> = {
            let mut seen = HashSet::new();
            colors.iter().filter(|&&color| seen.insert(color)).cloned().collect()
        };

        // Check for exact match first (cheapest check)
        let target_palette_array: [u8; COLORS_PER_TILE as usize] =
            from_fn(|i| if i < unique_colors.len() { unique_colors[i] } else { 0 });

        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            if *sub_pal == target_palette_array {
                // Create identity remapping for our original colors (including duplicates)
                let mut remapping = Vec::new();
                for &color in colors {
                    let unique_index = unique_colors.iter().position(|&c| c == color).unwrap_or(0);
                    remapping.push(unique_index as u8);
                }
                return (i as u8, remapping);
            }
        }

        // Try to find an existing sub-palette that contains all our unique colors
        let color_set: HashSet<u8> = unique_colors.iter().cloned().collect();
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            let pal_colors: HashSet<u8> =
                sub_pal.iter().filter(|&&c| c != 0 || sub_pal[0] == 0).cloned().collect();
            if color_set.is_subset(&pal_colors) {
                // Create remapping from our canonical indices to sub-palette indices
                let mut remapping = Vec::new();
                for &color in colors {
                    let sub_pal_index =
                        sub_pal.iter().position(|&pal_color| pal_color == color).unwrap_or(0);
                    remapping.push(sub_pal_index as u8);
                }
                return (i as u8, remapping);
            }
        }

        // Create new sub-palette with unique colors only
        if self.sub_palette_head >= SUBPALETTE_COUNT as usize {
            panic!("Sub-palette capacity {} exceeded", SUBPALETTE_COUNT);
        }

        self.sub_palettes.push(target_palette_array);
        let palette_id = self.sub_palette_head as u8;
        self.sub_palette_head += 1;

        // Set name
        let name = format!("{}_{}", self.palette.name, palette_id);
        self.sub_palette_name_hash.insert(target_palette_array, name);

        // Create identity remapping for our original colors (including duplicates)
        let mut remapping = Vec::new();
        for &color in colors {
            let unique_index = unique_colors.iter().position(|&c| c == color).unwrap_or(0);
            remapping.push(unique_index as u8);
        }

        (palette_id, remapping)
    }

    fn find_or_create_single_color_sub_palette(&mut self, color: u8) -> (u8, Vec<u8>) {
        // Create a simple sub-palette with just this color in position 0
        let target_palette_array: [u8; COLORS_PER_TILE as usize] = [color, 0, 0, 0];

        // Check if we already have this single-color sub-palette
        for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
            if sub_pal[0] == color && sub_pal[1] == 0 && sub_pal[2] == 0 && sub_pal[3] == 0 {
                return (i as u8, vec![0]); // All pixels map to index 0
            }
        }

        // Create new single-color sub-palette
        if self.sub_palette_head >= SUBPALETTE_COUNT as usize {
            panic!("Sub-palette capacity {} exceeded", SUBPALETTE_COUNT);
        }

        self.sub_palettes.push(target_palette_array);
        let palette_id = self.sub_palette_head as u8;
        self.sub_palette_head += 1;

        // Set name
        let name = format!("{}_{}", self.palette.name, palette_id);
        self.sub_palette_name_hash.insert(target_palette_array, name);

        (palette_id, vec![0]) // All pixels map to index 0
    }
}

fn create_canonical_tile(tile_data: &TileData) -> (CanonicalTile, Vec<u8>) {
    let mut canonical = [0u8; TILE_LEN];
    let mut color_mapping = Vec::new();
    let mut color_to_index = HashMap::new();

    for (i, &color) in tile_data.iter().enumerate() {
        let canonical_index = if let Some(&index) = color_to_index.get(&color) {
            index
        } else {
            let new_index = color_mapping.len() as u8;
            color_mapping.push(color);
            color_to_index.insert(color, new_index);
            new_index
        };
        canonical[i] = canonical_index;
    }

    (canonical, color_mapping)
}

fn transform_tile(tile: &TileData, flip_x: bool, flip_y: bool, rotation: bool) -> TileData {
    let mut result = [0u8; TILE_LEN];
    let size = TILE_SIZE as usize;

    for y in 0..size {
        for x in 0..size {
            let src_idx = y * size + x;
            let mut dst_x = x;
            let mut dst_y = y;

            if rotation {
                let temp = dst_x;
                dst_x = dst_y;
                dst_y = size - 1 - temp;
            }
            if flip_x {
                dst_x = size - 1 - dst_x;
            }
            if flip_y {
                dst_y = size - 1 - dst_y;
            }

            result[dst_y * size + dst_x] = tile[src_idx];
        }
    }
    result
}
