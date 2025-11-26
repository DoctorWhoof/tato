use core::array::from_fn;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;
type TileData = [u8; TILE_LEN];

// Colors remapped to canonical form (0,1,2,3...) to allow detection of palette-swapped tiles!
pub(crate) type CanonicalTile = [u8; TILE_LEN];

#[derive(Clone)]
enum DeferredCommand {
    NewGroup { path: String, name: String },
    NewTile { path: String },
    NewMap { path: String, name: String },
    NewAnimationStrip { path: String, name: String, frames_h: u8, frames_v: u8 },
}

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
    groups: &'a mut GroupBuilder,
    anims: Vec<AnimBuilder>,
    maps: Vec<MapBuilder>,
    single_tiles: Vec<SingleTileBuilder>,
    palette: &'a mut PaletteBuilder,
    next_tile: u8,
    sub_palette_head: usize,
    deferred_commands: Vec<DeferredCommand>,
}

impl<'a> TilesetBuilder<'a> {
    pub fn new(name: &str, palette: &'a mut PaletteBuilder, groups: &'a mut GroupBuilder) -> Self {
        crate::ensure_init_build();
        Self {
            allow_tile_transforms: true,
            allow_unused: false,
            use_crate_assets: false,
            save_colors: true,
            name: String::from(name),
            pixels: vec![],
            tile_hash: HashMap::new(),
            groups,
            sub_palette_name_hash: HashMap::new(),
            next_tile: 0,
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            palette,
            sub_palettes: Vec::new(),
            sub_palette_head: 0,
            deferred_commands: Vec::new(),
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

    fn should_regenerate_output(&self) -> bool {
        // Check if any deferred command file needs regeneration
        for command in &self.deferred_commands {
            let file_path = match command {
                DeferredCommand::NewGroup { path, .. } => path,
                DeferredCommand::NewTile { path } => path,
                DeferredCommand::NewMap { path, .. } => path,
                DeferredCommand::NewAnimationStrip { path, .. } => path,
            };
            if !file_path.is_empty() && crate::should_regenerate_file(file_path) {
                return true;
            }
        }
        false
    }

    fn execute_deferred_commands(&mut self) {
        let commands = self.deferred_commands.clone();
        for command in commands {
            match command {
                DeferredCommand::NewGroup { path, name } => {
                    let group_index = self.groups.add_group(&name);
                    if !path.is_empty() {
                        let img = self.load_valid_image(&path, 1, 1);
                        let _ = self.add_tiles(&img, Some(group_index));
                    }
                },
                DeferredCommand::NewTile { path } => {
                    let img = self.load_valid_image(&path, 1, 1);
                    assert!(
                        img.width == TILE_SIZE as usize,
                        "Single tile width must be {}",
                        TILE_SIZE
                    );
                    assert!(
                        img.cols_per_frame == 1 && img.rows_per_frame == 1,
                        "Single tile must be 1x1 tile (8x8 pixels)"
                    );
                    let cells = self.add_tiles(&img, None);
                    assert!(cells.len() == 1 && cells[0].len() == 1);
                    let tile_name = crate::strip_path_name(&path);
                    let single_tile =
                        SingleTileBuilder { name: tile_name, cell: cells[0][0].clone() };
                    self.single_tiles.push(single_tile);
                },
                DeferredCommand::NewMap { path, name } => {
                    let img = self.load_valid_image(&path, 1, 1);
                    let frames = self.add_tiles(&img, None);
                    assert!(frames.len() == 1);
                    let map = MapBuilder {
                        name,
                        columns: u8::try_from(img.cols_per_frame).unwrap(),
                        rows: u8::try_from(img.rows_per_frame).unwrap(),
                        cells: frames[0].clone(),
                    };
                    self.maps.push(map);
                },
                DeferredCommand::NewAnimationStrip { path, name, frames_h, frames_v } => {
                    let img = self.load_valid_image(&path, frames_h, frames_v);
                    let cells = self.add_tiles(&img, None);
                    let frame_count = img.frames_h as usize * img.frames_v as usize;
                    assert!(frame_count > 0);
                    let anim = AnimBuilder {
                        name: name.clone(),
                        frames: (0..frame_count)
                            .map(|i| MapBuilder {
                                name: format!("frame_{:02}", i),
                                columns: u8::try_from(img.cols_per_frame).unwrap(),
                                rows: u8::try_from(img.rows_per_frame).unwrap(),
                                cells: cells[i].clone(),
                            })
                            .collect(),
                    };
                    self.anims.push(anim);
                },
            }
        }
    }

    /// Defines a new tile group. Adds the tiles only, does not add Tilemaps or Animations,
    /// there's a match. To add an "empty" group, with no tiles, simply add it directly to
    /// the GroupBuilder instead of using the TilesetBuilder.
    pub fn new_group(&mut self, path: &str, name: &str) {
        self.deferred_commands
            .push(DeferredCommand::NewGroup { path: path.to_string(), name: name.to_string() });
    }

    /// Creates a new single tile from a .png file
    pub fn new_tile(&mut self, path: &str) {
        self.deferred_commands.push(DeferredCommand::NewTile { path: path.to_string() });
    }

    /// Creates a new map from a .png file
    pub fn new_map(&mut self, path: &str, name: &str) {
        self.deferred_commands
            .push(DeferredCommand::NewMap { path: path.to_string(), name: name.to_string() });
    }

    /// Creates a new animation strip from a .png file
    pub fn new_animation_strip(&mut self, path: &str, name: &str, frames_h: u8, frames_v: u8) {
        self.deferred_commands.push(DeferredCommand::NewAnimationStrip {
            path: path.to_string(),
            name: name.to_string(),
            frames_h,
            frames_v,
        });
    }

    /// Writes the tileset constants to a file
    pub fn write(&mut self, file_path: &str) {
        // Check if any input files have changed
        if !self.should_regenerate_output() {
            return;
        }

        println!("cargo:warning=Regenerating tileset: {}", file_path);

        // Execute all deferred commands now
        self.execute_deferred_commands();

        println!("cargo:warning=Creating output file: {}", file_path);
        let mut code = CodeWriter::new(file_path);

        // Write header
        code.write_header(self.allow_unused, self.use_crate_assets);

        // Write private mod statements for tilemaps at the top
        for map in &self.maps {
            code.write_line(&format!("mod {};", map.name.to_lowercase()));
        }
        if !self.maps.is_empty() {
            code.write_line("");
        }

        // Write re-exports for tilemap constants
        for map in &self.maps {
            code.write_line(&format!("pub use {}::*;", map.name.to_lowercase()));
        }
        if !self.maps.is_empty() {
            code.write_line("");
        }

        // Generate separate files for each tilemap
        self.write_tilemap_files(file_path);

        // Write tileset data structure
        code.write_line(&format!(
            "pub const {}_TILESET: TilesetData = TilesetData{{",
            self.name.to_uppercase(),
        ));
        if !self.pixels.is_empty() {
            code.write_line(&format!("    tiles: Some(&{}_TILES),", self.name.to_uppercase()));
        } else {
            code.write_line("    tiles: None,");
        }

        if self.save_colors {
            code.write_line(&format!("    colors: Some(&{}_COLORS),", self.name.to_uppercase()));
        } else {
            code.write_line("    colors: None,");
        }

        if self.save_colors && !self.sub_palettes.is_empty() {
            code.write_line(&format!("    sub_palettes: Some(&["));
            for i in 0..self.sub_palettes.len() {
                code.write_line(&format!(
                    "        &{}_SUBPALETTE_{},",
                    self.name.to_uppercase(),
                    i
                ));
            }
            code.write_line("    ]),");
        } else {
            code.write_line("    sub_palettes: None,");
        }

        code.write_line("};");
        code.write_line("");

        // Write palette colors
        if self.save_colors {
            code.write_color_array(&self.name, &self.palette.colors);
        }

        // Write sub-palettes
        if self.save_colors {
            for (i, sub_palette) in self.sub_palettes.iter().enumerate() {
                code.write_line(&format!(
                    "#[unsafe(link_section = \"{}\")]",
                    crate::get_platform_link_section()
                ));
                code.write_line(&format!(
                    "pub static {}_SUBPALETTE_{}: [u8; {}] = [",
                    self.name.to_uppercase(),
                    i,
                    sub_palette.len()
                ));

                for &color_index in sub_palette {
                    code.write_line(&format!("    {},", color_index));
                }

                code.write_line("];");
                code.write_line("");
            }
        }

        // Write animation strips if any
        if !self.anims.is_empty() {
            for anim in &self.anims {
                code.write_line(&format!(
                    "#[unsafe(link_section = \"{}\")]",
                    crate::get_platform_link_section()
                ));
                code.write_line(&format!(
                    "pub static {}: [Tilemap<{}>; {}] = [",
                    anim.name.to_uppercase(),
                    anim.frames[0].cells.len(),
                    anim.frames.len()
                ));

                for frame in &anim.frames {
                    code.write_line("    Tilemap {");
                    code.write_line("        cells: [");

                    for cell in &frame.cells {
                        code.write_cell(cell);
                    }

                    code.write_line("        ],");
                    code.write_line(&format!("        columns: {},", frame.columns));
                    code.write_line(&format!("        rows: {},", frame.rows));
                    code.write_line("    },");
                }

                code.write_line("];");
                code.write_line("");
            }
        }

        // Write single tiles
        for tile in &self.single_tiles {
            if self.name == "default" {
                // For default tileset, generate TileID constants for type safety
                code.write_line(&format!(
                    "pub const {}: TileID = TileID({});",
                    tile.name.to_uppercase(),
                    tile.cell.id.0
                ));
            } else {
                code.write_line(&format!(
                    "pub const {}: Cell = {};",
                    tile.name.to_uppercase(),
                    crate::format_cell_compact(&tile.cell)
                ));
            }
        }
        if !self.single_tiles.is_empty() {
            code.write_line("");
        }

        // Write tile pixel data
        if !self.pixels.is_empty() {
            let tiles_count = self.pixels.len() / (TILE_SIZE as usize * TILE_SIZE as usize);
            code.write_line(&format!(
                "#[unsafe(link_section = \"{}\")]",
                crate::get_platform_link_section()
            ));
            code.write_line(&format!(
                "pub static {}_TILES: [Tile<2>; {}] = [",
                self.name.to_uppercase(),
                tiles_count
            ));

            for i in 0..tiles_count {
                let start = i * (TILE_SIZE as usize * TILE_SIZE as usize);
                let end = start + (TILE_SIZE as usize * TILE_SIZE as usize);
                let tile_pixels = &self.pixels[start..end];

                code.write_line(&format!("    {},", crate::format_tile_compact(tile_pixels)));
            }
            code.write_line("];");
        }

        // Format the output
        println!("cargo:warning=Formatting and writing file: {}", file_path);
        code.format_output(file_path);
        println!("cargo:warning=File write completed: {}", file_path);

        // Mark all input files as processed
        for command in &self.deferred_commands {
            let file_path = match command {
                DeferredCommand::NewGroup { path, .. } => path,
                DeferredCommand::NewTile { path } => path,
                DeferredCommand::NewMap { path, .. } => path,
                DeferredCommand::NewAnimationStrip { path, .. } => path,
            };
            if !file_path.is_empty() {
                crate::mark_file_processed(file_path);
            }
        }
    }

    fn write_tilemap_files(&self, main_file_path: &str) {
        use std::fs;
        use std::path::Path;

        // Get the directory of the main file and the module name
        let main_path = Path::new(main_file_path);
        let output_dir = main_path.parent().expect("Could not get parent directory of output file");
        let module_name = main_path
            .file_stem()
            .expect("Could not get module name from file path")
            .to_str()
            .expect("Could not convert module name to string");

        for map in &self.maps {
            // Create subdirectory for this module
            let module_subdir = output_dir.join(module_name);
            if let Err(e) = fs::create_dir_all(&module_subdir) {
                println!("cargo:warning=Failed to create directory {:?}: {}", module_subdir, e);
                continue;
            }

            let map_filename = format!("{}.rs", map.name.to_lowercase());
            let map_file_path = module_subdir.join(&map_filename);
            let map_file_path_str =
                map_file_path.to_str().expect("Could not convert path to string");

            println!("cargo:warning=Creating tilemap file: {}", map_file_path_str);
            let mut code = CodeWriter::new(map_file_path_str);

            // Write header
            code.write_header(self.allow_unused, self.use_crate_assets);

            // Write the tilemap
            code.write_line(&format!(
                "#[unsafe(link_section = \"{}\")]",
                crate::get_platform_link_section()
            ));
            code.write_line(&format!(
                "pub static {}: Tilemap<{}> = Tilemap {{",
                map.name.to_uppercase(),
                map.cells.len()
            ));
            code.write_line(&format!("    columns: {},", map.columns));
            code.write_line(&format!("    rows: {},", map.rows));
            code.write_line("    cells: [");

            for cell in &map.cells {
                code.write_cell(cell);
            }

            code.write_line("    ],");
            code.write_line("};");

            // Format the output
            println!("cargo:warning=Formatting and writing tilemap file: {}", map_file_path_str);
            code.format_output(map_file_path_str);
            println!("cargo:warning=Tilemap file write completed: {}", map_file_path_str);
        }
    }

    fn add_tiles(&mut self, img: &PalettizedImg, group: Option<u8>) -> Vec<Vec<Cell>> {
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
                            // Only register multi-color tiles in groups (skip empty/solid-color tiles)
                            if color_mapping.len() > 1 {
                                self.groups.register_tile(canonical_tile, group_idx);

                                // Also register all transformations if enabled
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                if !flip_x && !flip_y && !rotation {
                                                    continue; // Skip identity transform
                                                }

                                                let transformed_tile = transform_tile(
                                                    &tile_data, flip_x, flip_y, rotation,
                                                );
                                                let (transformed_canonical, transformed_colors) =
                                                    create_canonical_tile(&transformed_tile);

                                                // Only register if the transformed tile is also multi-color
                                                if transformed_colors.len() > 1 {
                                                    self.groups.register_tile(
                                                        transformed_canonical,
                                                        group_idx,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Safety check
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

                        // Handle solid-color tiles efficiently
                        let (sub_palette_id, remapping) = if color_mapping.len() <= 1 {
                            // Solid color tile - find or create a simple sub-palette
                            let color = color_mapping.get(0).copied().unwrap_or(0);

                            // Create a simple sub-palette with just this color in position 0
                            let target_palette_array: [u8; COLORS_PER_TILE as usize] =
                                [color, 0, 0, 0];

                            // Check if we already have this solid-color sub-palette
                            let mut found_palette_id = None;
                            for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
                                if sub_pal[0] == color
                                    && sub_pal[1] == 0
                                    && sub_pal[2] == 0
                                    && sub_pal[3] == 0
                                {
                                    found_palette_id = Some(i as u8);
                                    break;
                                }
                            }

                            if let Some(palette_id) = found_palette_id {
                                (palette_id, vec![0]) // All pixels map to index 0
                            } else {
                                // Create new solid-color sub-palette
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
                        } else {
                            // Multi-color tile - use normal processing
                            // Work with unique colors only to avoid issues with repeated colors
                            let mut unique_colors: Vec<u8> = {
                                let mut seen = HashSet::new();
                                color_mapping
                                    .iter()
                                    .filter(|&&color| seen.insert(color))
                                    .cloned()
                                    .collect()
                            };
                            unique_colors.sort_unstable();

                            // Check for exact match first (cheapest check)
                            let target_palette_array: [u8; COLORS_PER_TILE as usize] = from_fn(
                                |i| if i < unique_colors.len() { unique_colors[i] } else { 0 },
                            );

                            let mut found_palette_result = None;
                            for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
                                if *sub_pal == target_palette_array {
                                    // Create identity remapping for our original colors (including duplicates)
                                    let mut remapping = Vec::new();
                                    for &color in &color_mapping {
                                        let unique_index = unique_colors
                                            .iter()
                                            .position(|&c| c == color)
                                            .unwrap_or(0);
                                        remapping.push(unique_index as u8);
                                    }
                                    found_palette_result = Some((i as u8, remapping));
                                    break;
                                }
                            }

                            if let Some(result) = found_palette_result {
                                result
                            } else {
                                // Try to find an existing sub-palette that contains all our unique colors
                                let color_set: HashSet<u8> =
                                    unique_colors.iter().cloned().collect();
                                let mut found_compatible_result = None;
                                for (i, sub_pal) in self.sub_palettes.iter().enumerate() {
                                    let pal_colors: HashSet<u8> = sub_pal
                                        .iter()
                                        .filter(|&&c| c != 0 || sub_pal[0] == 0)
                                        .cloned()
                                        .collect();
                                    if color_set.is_subset(&pal_colors) {
                                        // Create remapping from our canonical indices to sub-palette indices
                                        let mut remapping = Vec::new();
                                        for &color in &color_mapping {
                                            let sub_pal_index = sub_pal
                                                .iter()
                                                .position(|&pal_color| pal_color == color)
                                                .unwrap_or(0);
                                            remapping.push(sub_pal_index as u8);
                                        }
                                        found_compatible_result = Some((i as u8, remapping));
                                        break;
                                    }
                                }

                                if let Some(result) = found_compatible_result {
                                    result
                                } else {
                                    // Create new sub-palette with unique colors only
                                    if self.sub_palette_head >= SUBPALETTE_COUNT as usize {
                                        panic!(
                                            "Sub-palette capacity {} exceeded",
                                            SUBPALETTE_COUNT
                                        );
                                    }

                                    self.sub_palettes.push(target_palette_array);
                                    let palette_id = self.sub_palette_head as u8;
                                    self.sub_palette_head += 1;

                                    // Set name
                                    let name = format!("{}_{}", self.palette.name, palette_id);
                                    self.sub_palette_name_hash.insert(target_palette_array, name);

                                    // Create identity remapping for our original colors (including duplicates)
                                    let mut remapping = Vec::new();
                                    for &color in &color_mapping {
                                        let unique_index = unique_colors
                                            .iter()
                                            .position(|&c| c == color)
                                            .unwrap_or(0);
                                        remapping.push(unique_index as u8);
                                    }

                                    (palette_id, remapping)
                                }
                            }
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
                        let group_bits =
                            self.groups.hash.get(&canonical_tile).copied().unwrap_or(0);

                        let cell = match found_cell {
                            Some(existing_cell) => {
                                // Found existing tile with same pattern
                                // Use the same sub-palette mapping we used for lookup
                                Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id),
                                }
                            },
                            None => {
                                // Create new tile using the sub-palette we already found/created
                                let new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id),
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
}

fn create_canonical_tile(tile_data: &TileData) -> (CanonicalTile, Vec<u8>) {
    let mut canonical = [0u8; TILE_LEN];
    let mut color_mapping = Vec::new();
    let mut color_to_index = HashMap::new();

    // First, collect all unique colors
    let mut unique_colors = Vec::new();
    for &color in tile_data.iter() {
        if !color_to_index.contains_key(&color) {
            unique_colors.push(color);
            color_to_index.insert(color, 0); // Temporary placeholder
        }
    }

    // Sort colors by their palette index to maintain consistent ordering
    // This ensures 0=darkest, 1=darker, 2=lighter, 3=lightest
    unique_colors.sort_unstable();

    // Assign canonical indices based on sorted order
    color_to_index.clear();
    for (canonical_index, &color) in unique_colors.iter().enumerate() {
        color_mapping.push(color);
        color_to_index.insert(color, canonical_index as u8);
    }

    // Map each pixel to its canonical index
    for (i, &color) in tile_data.iter().enumerate() {
        canonical[i] = color_to_index[&color];
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
