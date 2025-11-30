use tato_video::*;

use super::*;
use super::subpalette::SubPaletteBuilder;
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
    sub_palette_builders: Vec<SubPaletteBuilder>,
    sub_palettes_with_tiles: Vec<bool>, // Track which sub-palettes have tiles stored
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
            sub_palette_builders: Vec::new(),
            sub_palettes_with_tiles: Vec::new(),
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
                        let tiles_before = self.next_tile;
                        let img = self.load_valid_image(&path, 1, 1);
                        let _ = self.add_tiles(&img, Some(group_index));
                        let tiles_added = self.next_tile - tiles_before;
                        println!("cargo:warning=NEW_GROUP '{}': Added {} tiles (total: {})", name, tiles_added, self.next_tile);
                    }
                },
                DeferredCommand::NewTile { path } => {
                    let tiles_before = self.next_tile;
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
                        SingleTileBuilder { name: tile_name.clone(), cell: cells[0][0].clone() };
                    self.single_tiles.push(single_tile);
                    let tiles_added = self.next_tile - tiles_before;
                    println!("cargo:warning=NEW_TILE '{}': Added {} tiles (total: {})", tile_name, tiles_added, self.next_tile);
                },
                DeferredCommand::NewMap { path, name } => {
                    let tiles_before = self.next_tile;
                    let img = self.load_valid_image(&path, 1, 1);
                    let frames = self.add_tiles(&img, None);
                    assert!(frames.len() == 1);
                    let map = MapBuilder {
                        name: name.clone(),
                        columns: u8::try_from(img.cols_per_frame).unwrap(),
                        rows: u8::try_from(img.rows_per_frame).unwrap(),
                        cells: frames[0].clone(),
                    };
                    self.maps.push(map);
                    let tiles_added = self.next_tile - tiles_before;
                    println!("cargo:warning=NEW_MAP '{}': Added {} tiles (total: {}), image size: {}x{} tiles", 
                             name, tiles_added, self.next_tile, img.cols_per_frame, img.rows_per_frame);
                },
                DeferredCommand::NewAnimationStrip { path, name, frames_h, frames_v } => {
                    let tiles_before = self.next_tile;
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
                    let tiles_added = self.next_tile - tiles_before;
                    println!("cargo:warning=NEW_ANIMATION_STRIP '{}': Added {} tiles (total: {}), frames: {}x{}, tiles per frame: {}x{}", 
                             name, tiles_added, self.next_tile, frames_h, frames_v, img.cols_per_frame, img.rows_per_frame);
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

        // Optimize sub-palettes after all tiles are processed
        // DISABLED: Causes color mapping issues, needs debugging
        // self.consolidate_sub_palettes();

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

        // Print final tile summary
        let total_pixels = self.pixels.len();
        let tiles_per_tile = TILE_SIZE as usize * TILE_SIZE as usize;
        let total_tiles = total_pixels / tiles_per_tile;
        println!("cargo:warning=FINAL SUMMARY: {} unique tiles created, {} pixels stored, {} sub-palettes used", 
                 total_tiles, total_pixels, self.sub_palettes.len());
        
        if total_tiles != self.next_tile as usize {
            println!("cargo:warning=WARNING: Tile count inconsistency - next_tile: {}, actual tiles: {}", 
                     self.next_tile, total_tiles);
        }

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
        let tiles_before = self.next_tile;
        let mut frames = vec![];
        let mut tiles_created = 0;
        let mut tiles_reused = 0;

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
                            // Solid-color tile - find or create a simple sub-palette
                            let color = color_mapping.get(0).copied().unwrap_or(0);
                            let needed_builder = SubPaletteBuilder::from_colors(&[color]);

                            // Try to find an existing sub-palette that can accommodate this color
                            let mut found_palette_id = None;
                            for (i, builder) in self.sub_palette_builders.iter().enumerate() {
                                if builder.contains_all(&[color]) {
                                    found_palette_id = Some(i as u8);
                                    break;
                                }
                            }

                            if let Some(palette_id) = found_palette_id {
                                if palette_id as usize >= self.sub_palette_builders.len() {
                                    panic!("Invalid palette_id {} for {} builders", palette_id, self.sub_palette_builders.len());
                                }
                                let remapping = self.sub_palette_builders[palette_id as usize]
                                    .create_remapping(&[color])
                                    .unwrap_or_else(|| {
                                        eprintln!("WARNING: Failed to create remapping for existing solid-color palette {}", palette_id);
                                        vec![0]
                                    });
                                (palette_id, remapping)
                            } else {
                                // Try to merge with an existing sub-palette that has space AND no tiles stored yet
                                let mut merge_candidate = None;
                                for (i, builder) in self.sub_palette_builders.iter().enumerate() {
                                    if builder.can_merge(&needed_builder) && !self.sub_palettes_with_tiles[i] {
                                        merge_candidate = Some(i);
                                        break;
                                    }
                                }

                                if let Some(merge_idx) = merge_candidate {
                                    if merge_idx >= self.sub_palette_builders.len() || merge_idx >= self.sub_palettes.len() {
                                        panic!("Invalid merge_idx {} for solid-color merge (builders: {}, palettes: {})", 
                                               merge_idx, self.sub_palette_builders.len(), self.sub_palettes.len());
                                    }
                                    
                                    // SAFE: Merge with existing sub-palette that has no tiles stored yet
                                    let old_builder = self.sub_palette_builders[merge_idx].clone();
                                    println!("cargo:warning=  SAFE MERGE: solid-color sub-palette {} {:?} + {:?}", 
                                             merge_idx, old_builder.colors(), needed_builder.colors());
                                    let merged = old_builder.merge(needed_builder).unwrap();
                                    println!("cargo:warning=    Result: {:?}", merged.colors());
                                    
                                    // Update the existing sub-palette since no tiles reference it yet
                                    self.sub_palette_builders[merge_idx] = merged.clone();
                                    self.sub_palettes[merge_idx] = merged.to_array();
                                    
                                    // Update name
                                    let name = format!("{}_{}", self.palette.name, merge_idx);
                                    self.sub_palette_name_hash.insert(merged.to_array(), name);
                                    
                                    let remapping = merged.create_remapping(&[color]).unwrap_or_else(|| {
                                        eprintln!("WARNING: Failed to create remapping for merged solid-color palette {}", merge_idx);
                                        vec![0]
                                    });
                                    (merge_idx as u8, remapping)
                                } else {
                                    // Create new solid-color sub-palette
                                    if self.sub_palette_head >= SUBPALETTE_COUNT as usize {
                                        panic!("Sub-palette capacity {} exceeded", SUBPALETTE_COUNT);
                                    }

                                    let target_palette_array = needed_builder.to_array();
                                    let palette_id = self.sub_palettes.len();

                                    if palette_id >= SUBPALETTE_COUNT as usize {
                                        panic!("Sub-palette index {} exceeds maximum {}", palette_id, SUBPALETTE_COUNT);
                                    }

                                    self.sub_palettes.push(target_palette_array);
                                    self.sub_palette_builders.push(needed_builder.clone());
                                    self.sub_palettes_with_tiles.push(false);
                                    self.sub_palette_head += 1;

                                    // Set name
                                    let name = format!("{}_{}", self.palette.name, palette_id);
                                    self.sub_palette_name_hash.insert(target_palette_array, name);

                                    let remapping = needed_builder.create_remapping(&[color]).unwrap_or_else(|| {
                                        eprintln!("WARNING: Failed to create remapping for color {} in solid-color tile", color);
                                        vec![0]
                                    });
                                    (palette_id as u8, remapping)
                                }
                            }
                        } else {
                            // Multi-color tile - use SubPaletteBuilder system for optimal reuse

                            // Work with unique colors only to avoid issues with repeated colors
                            // IMPORTANT: Preserve the original sorted order from color_mapping
                            let unique_colors: Vec<u8> = {
                                let mut unique = Vec::new();
                                let mut seen = HashSet::new();
                                for &color in &color_mapping {
                                    if seen.insert(color) {
                                        unique.push(color);
                                    }
                                }
                                unique
                            };

                            let needed_builder = SubPaletteBuilder::from_colors(&unique_colors);

                            // First try to find an existing sub-palette that contains all our colors
                            let mut found_compatible = None;
                            for (i, builder) in self.sub_palette_builders.iter().enumerate() {
                                if builder.contains_all(&unique_colors) {
                                    found_compatible = Some(i);
                                    break;
                                }
                            }

                            if let Some(compatible_idx) = found_compatible {
                                if compatible_idx >= self.sub_palette_builders.len() {
                                    panic!("Invalid compatible_idx {} for {} builders", compatible_idx, self.sub_palette_builders.len());
                                }
                                // Use existing compatible sub-palette
                                let builder = &self.sub_palette_builders[compatible_idx];
                                let remapping = builder.create_remapping(&color_mapping).unwrap_or_else(|| {
                                    eprintln!("WARNING: Failed to create remapping for compatible palette {}, using fallback", compatible_idx);
                                    // Fallback: create remapping based on unique colors
                                    color_mapping.iter().map(|&color| {
                                        builder.colors().iter().position(|&c| c == color).unwrap_or(0) as u8
                                    }).collect()
                                });
                                (compatible_idx as u8, remapping)
                            } else {
                                // Try to merge with an existing sub-palette that has no tiles stored yet
                                let mut merge_candidate = None;
                                for (i, builder) in self.sub_palette_builders.iter().enumerate() {
                                    if builder.can_merge(&needed_builder) && !self.sub_palettes_with_tiles[i] {
                                        merge_candidate = Some(i);
                                        break;
                                    }
                                }

                                if let Some(merge_idx) = merge_candidate {
                                    if merge_idx >= self.sub_palette_builders.len() || merge_idx >= self.sub_palettes.len() {
                                        panic!("Invalid merge_idx {} for {} builders/{} palettes", 
                                               merge_idx, self.sub_palette_builders.len(), self.sub_palettes.len());
                                    }
                                    
                                    // SAFE: Merge with existing sub-palette that has no tiles stored yet
                                    let old_builder = self.sub_palette_builders[merge_idx].clone();
                                    println!("cargo:warning=  SAFE MERGE: multi-color sub-palette {} {:?} + {:?}", 
                                             merge_idx, old_builder.colors(), needed_builder.colors());
                                    let merged = old_builder.merge(needed_builder).unwrap();
                                    println!("cargo:warning=    Result: {:?}", merged.colors());
                                    
                                    // Update the existing sub-palette since no tiles reference it yet
                                    self.sub_palette_builders[merge_idx] = merged.clone();
                                    self.sub_palettes[merge_idx] = merged.to_array();
                                    
                                    // Update name
                                    let name = format!("{}_{}", self.palette.name, merge_idx);
                                    self.sub_palette_name_hash.insert(merged.to_array(), name);
                                    
                                    let remapping = merged.create_remapping(&color_mapping).unwrap_or_else(|| {
                                        eprintln!("WARNING: Failed to create remapping for merged multi-color palette {}, using fallback", merge_idx);
                                        // Fallback: create remapping based on color positions
                                        color_mapping.iter().map(|&color| {
                                            merged.colors().iter().position(|&c| c == color).unwrap_or(0) as u8
                                        }).collect()
                                    });
                                    (merge_idx as u8, remapping)
                                } else {
                                    // Create new sub-palette
                                    if self.sub_palette_head >= SUBPALETTE_COUNT as usize {
                                        panic!(
                                            "Sub-palette capacity {} exceeded",
                                            SUBPALETTE_COUNT
                                        );
                                    }

                                    let target_palette_array = needed_builder.to_array();
                                    let palette_id = self.sub_palettes.len();

                                    if palette_id >= SUBPALETTE_COUNT as usize {
                                        panic!("Sub-palette index {} exceeds maximum {}", palette_id, SUBPALETTE_COUNT);
                                    }

                                    self.sub_palettes.push(target_palette_array);
                                    self.sub_palette_builders.push(needed_builder.clone());
                                    self.sub_palettes_with_tiles.push(false);
                                    self.sub_palette_head += 1;

                                    // Set name
                                    let name = format!("{}_{}", self.palette.name, palette_id);
                                    self.sub_palette_name_hash.insert(target_palette_array, name);

                                    let remapping = needed_builder.create_remapping(&color_mapping).unwrap_or_else(|| {
                                        eprintln!("WARNING: Failed to create remapping for new multi-color palette, using fallback");
                                        // Fallback: create remapping for duplicated colors
                                        color_mapping.iter().map(|&color| {
                                            needed_builder.colors().iter().position(|&c| c == color).unwrap_or(0) as u8
                                        }).collect()
                                    });
                                    (palette_id as u8, remapping)
                                }
                            }
                        };



                        // Check if this canonical tile (or any transformation) exists
                        let mut found_cell = None;
                        let mut normalized_tile = [0u8; TILE_LEN];
                        for (i, &canonical_index) in canonical_tile.iter().enumerate() {
                            if (canonical_index as usize) < remapping.len() {
                                normalized_tile[i] = remapping[canonical_index as usize];
                            } else {
                                // This should never happen if remapping is created correctly
                                eprintln!("WARNING: canonical_index {} >= remapping.len() {}", canonical_index, remapping.len());
                                normalized_tile[i] = 0;
            }
        }

                        // Check original first using remapped data
                        if let Some(existing) = self.tile_hash.get(&normalized_tile) {
                            found_cell = Some(*existing);
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations - check against ALL existing sub-palettes
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

                                        // Try remapping with each existing sub-palette
                                        for (_existing_sub_palette_id, existing_builder) in self.sub_palette_builders.iter().enumerate() {
                                            // Check if this sub-palette contains all the transformed colors
                                            if !existing_builder.contains_all(&transformed_colors) {
                                                continue;
                                            }

                                            // Apply remapping using this existing sub-palette
                                            let mut transformed_normalized = [0u8; TILE_LEN];
                                            let mut mapping_failed = false;
                                            
                                            for (i, &canonical_index) in transformed_canonical.iter().enumerate() {
                                                if (canonical_index as usize) < transformed_colors.len() {
                                                    let color = transformed_colors[canonical_index as usize];
                                                    match existing_builder.colors().iter().position(|&c| c == color) {
                                                        Some(sub_palette_index) => {
                                                            transformed_normalized[i] = sub_palette_index as u8;
                                                        }
                                                        None => {
                                                            mapping_failed = true;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    transformed_normalized[i] = 0;
                                                }
                                            }

                                            if !mapping_failed {
                                                if let Some(existing) = self.tile_hash.get(&transformed_normalized) {
                                                    println!("cargo:warning=  Found transform match: TileID({}) with flip_x={}, flip_y={}, rotation={}", 
                                                             existing.id.0, flip_x, flip_y, rotation);
                                                    found_cell = Some(*existing);
                                                    break 'outer;
                                                }
                                            }
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
                                tiles_reused += 1;
                                println!("cargo:warning=  Reusing TileID({}) at position ({}, {}) in frame ({}, {})", 
                                         existing_cell.id.0, col, row, frame_h, frame_v);
                                Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id),
                                }
                            },
                            None => {
                                // Create new tile using the sub-palette we already found/created
                                tiles_created += 1;
                                println!("cargo:warning=  Creating NEW TileID({}) at position ({}, {}) in frame ({}, {}) - colors: {:?}", 
                                         self.next_tile, col, row, frame_h, frame_v, color_mapping);
                                let new_tile = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                    group: group_bits,
                                    sub_palette: PaletteID(sub_palette_id),
                                };

                                // Store the already computed normalized_tile tile data
                                self.pixels.extend_from_slice(&normalized_tile);
                                
                                // Mark this sub-palette as having tiles stored
                                if (sub_palette_id as usize) < self.sub_palettes_with_tiles.len() {
                                    self.sub_palettes_with_tiles[sub_palette_id as usize] = true;
                                }

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
                                                    if (canonical_index as usize) < transformed_colors.len() {
                                                        let color = transformed_colors[canonical_index as usize];
                                                        // Find this color in our sub-palette builder
                                                        let sub_palette_index = self.sub_palette_builders[sub_palette_id as usize]
                                                            .colors()
                                                            .iter()
                                                            .position(|&c| c == color)
                                                            .unwrap_or(0) as u8;
                                                        transformed_normalized[i] = sub_palette_index;
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

        let tiles_after = self.next_tile;
        let total_tiles_added = tiles_after - tiles_before;
        
        if total_tiles_added > 0 || tiles_reused > 0 {
            println!("cargo:warning=  Tile processing: {} tiles created, {} tiles reused, {} total unique tiles now", 
                     total_tiles_added, tiles_reused, tiles_after);
            
            if tiles_created != total_tiles_added {
                println!("cargo:warning=  WARNING: tile count mismatch - created: {}, actual added: {}", 
                         tiles_created, total_tiles_added);
            }
        }
        
        frames
    }

    /// Consolidate sub-palettes after all tiles are processed
    /// DISABLED: Currently causes incorrect color mapping
    #[allow(dead_code)]
    fn consolidate_sub_palettes(&mut self) {
        // This function needs to be redesigned to properly handle pixel remapping
        // Current issues:
        // 1. Pixel remapping logic is flawed
        // 2. tile_hash lookup by pixel comparison is unreliable  
        // 3. Need to preserve exact color-to-index mapping
        
        println!("cargo:warning=Sub-palette consolidation DISABLED - would cause color mapping errors");
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
