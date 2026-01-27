use super::Anim;
use tato_video::*;

use super::*;
use crate::*;
use std::collections::HashMap;

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;

// Canonical tiles store the pixel "structure", not the
// colors themselves. Useful to detect palette swapped tiles!
#[derive(Debug, Clone)]
pub(crate) struct CanonicalTile {
    pub pixels: [u8; TILE_LEN],
    pub mapping: Vec<u8>,
}

// Allows a single pass executing commands when writing
#[derive(Clone)]
enum DeferredCommand {
    NewEmptyTile,
    NewTile { path: String },
    NewGroup { path: String, name: String },
    NewMap { path: String, name: String },
    NewAnimationStrip { path: String, name: String, frames_h: u8, frames_v: u8 },
    NewAnim { name: String, fps: u8, repeat: bool, strip_name: String, frames: Vec<u8> },
}

/// Converts images into deduplicated tiles, maps, and animations.
pub struct BankBuilder<'a> {
    /// If true, detects flipped/rotated tile variants to reduce tile count.
    pub allow_tile_transforms: bool,
    /// If true, allows unused warnings in generated code.
    pub allow_unused: bool,
    /// If true, writes color palette data.
    pub write_colors: bool,
    /// If true, writes tile pixel data.
    pub write_tiles: bool,
    /// If true, writes animation data.
    pub write_animations: bool,
    #[doc(hidden)]
    pub use_crate_assets: bool,
    name: String,
    pixels: Vec<u8>,
    tiles_to_cells: HashMap<Pixels, Cell>,
    // Store canonical tile info for each pattern
    canonical_tiles: HashMap<Pixels, CanonicalTile>,
    // Store all unique color mappings (index 0 is always identity mapping)
    color_mappings: Vec<[u8; COLORS_PER_PALETTE as usize]>,
    palette: &'a mut PaletteBuilder,
    groups: &'a mut GroupBuilder,
    strips: HashMap<String, StripBuilder>,
    anims: Vec<Anim>,
    maps: Vec<MapBuilder>,
    single_tiles: Vec<SingleTileBuilder>,
    next_tile: u8,
    deferred_commands: Vec<DeferredCommand>,
}

impl<'a> BankBuilder<'a> {
    /// Creates a new bank builder with the given name, palette, and group references.
    pub fn new(name: &str, palette: &'a mut PaletteBuilder, groups: &'a mut GroupBuilder) -> Self {
        crate::ensure_init_build();
        Self {
            allow_tile_transforms: true,
            allow_unused: false,
            use_crate_assets: false,
            write_colors: true,
            write_tiles: true,
            write_animations: true,
            name: String::from(name),
            pixels: vec![],
            tiles_to_cells: HashMap::new(),
            canonical_tiles: HashMap::new(),
            // Initialize with identity mapping at index 0
            color_mappings: vec![core::array::from_fn(|i| i as u8)],
            palette,
            groups,
            strips: HashMap::new(),
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            next_tile: 0,
            deferred_commands: vec![],
        }
    }

    /// Adds tiles from image to a named group. Use empty path for group without tiles.
    pub fn new_group(&mut self, path: &str, name: &str) {
        self.deferred_commands
            .push(DeferredCommand::NewGroup { path: path.to_string(), name: name.to_string() });
    }

    /// Adds a single 8x8 tile from a PNG file.
    pub fn new_tile(&mut self, path: &str) {
        self.deferred_commands.push(DeferredCommand::NewTile { path: path.to_string() });
    }

    /// Adds an empty (transparent) tile.
    pub fn new_empty_tile(&mut self) {
        self.deferred_commands.push(DeferredCommand::NewEmptyTile);
    }

    /// Adds a tilemap from a PNG file.
    pub fn new_map(&mut self, path: &str, name: &str) {
        self.deferred_commands
            .push(DeferredCommand::NewMap { path: path.to_string(), name: name.to_string() });
    }

    /// Adds an animation strip from a PNG file with specified frame layout.
    pub fn new_animation_strip(&mut self, path: &str, name: &str, frames_h: u8, frames_v: u8) {
        self.deferred_commands.push(DeferredCommand::NewAnimationStrip {
            path: path.to_string(),
            name: name.to_string(),
            frames_h,
            frames_v,
        });
    }

    /// Defines an animation using frames from a strip.
    pub fn new_anim<const LEN: usize>(
        &mut self,
        anim_name: &str,
        strip_name: &str,
        fps: u8,
        repeat: bool,
        frames: [u8; LEN],
    ) {
        self.deferred_commands.push(DeferredCommand::NewAnim {
            name: anim_name.into(),
            fps,
            repeat,
            strip_name: strip_name.into(),
            frames: frames.into(),
        });
    }

    /// Writes the bank constants to a file relative to export path. Skipped if empty.
    /// Generates Rust code for all registered assets. Skips if sources unchanged.
    pub fn write(&mut self, file_path: &str) {
        // Make file_path relative to export path
        let settings = crate::get_build_settings();
        let full_path = std::path::Path::new(&settings.asset_export_path)
            .join(file_path)
            .to_str()
            .expect("Could not convert path to string")
            .to_string();

        // Check if any input files have changed
        let mut should_regenerate = false;
        for command in &self.deferred_commands {
            let file_path = match command {
                DeferredCommand::NewEmptyTile | DeferredCommand::NewAnim { .. } => "",
                DeferredCommand::NewGroup { path, .. } => path,
                DeferredCommand::NewTile { path } => path,
                DeferredCommand::NewMap { path, .. } => path,
                DeferredCommand::NewAnimationStrip { path, .. } => path,
            };
            // Check if any deferred command file needs regeneration
            if !file_path.is_empty() && crate::should_regenerate_file(file_path) {
                should_regenerate = true;
                break;
            }
        }
        if !should_regenerate {
            return;
        }

        println!("cargo:warning=Regenerating bank: {}", full_path);
        // Execute all deferred commands now
        {
            let commands = self.deferred_commands.clone();
            for command in commands {
                match command {
                    DeferredCommand::NewEmptyTile => {
                        let img = PalettizedImg::empty(self.palette);
                        let cells = self.add_tiles(&img, None);
                        assert!(cells.len() == 1 && cells[0].len() == 1);
                        let tile_name = "EMPTY".to_string();
                        let single_tile =
                            SingleTileBuilder { name: tile_name, cell: cells[0][0].clone() };
                        self.single_tiles.push(single_tile);
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
                    DeferredCommand::NewGroup { path, name } => {
                        let group_index = self.groups.add_group(&name);
                        if !path.is_empty() {
                            let img = self.load_valid_image(&path, 1, 1);
                            let _ = self.add_tiles(&img, Some(group_index));
                        }
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
                        let strip = StripBuilder {
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
                        self.strips.insert(name, strip);
                    },
                    DeferredCommand::NewAnim { name, fps, repeat, strip_name, frames } => {
                        if self.anims.len() == 255 {
                            panic!("BankBuilder: animation capacity of 256 reached");
                        }

                        let Some(strip) = self.strips.get(&strip_name) else {
                            panic!("BankBuilder: Can't find strip name {}", strip_name)
                        };

                        // Validate
                        for frame in &frames {
                            if *frame as usize >= strip.frames.len() {
                                panic!(
                                    "BankBuilder: Invalid Anim frame number '{}' on sequence {:?}",
                                    *frame, frames
                                );
                            }
                        }

                        self.anims.push(Anim {
                            name,
                            fps,
                            repeat,
                            frames: frames.into(),
                            strip_name: strip_name.into(),
                        })
                    },
                }
            }
        }

        println!("cargo:warning=Creating output file: {}", full_path);
        let mut code = CodeWriter::new(&full_path);

        // Write header
        let default_imports = self.write_animations | self.write_colors | self.write_tiles;
        code.write_header(self.allow_unused, self.use_crate_assets, default_imports);

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

        // Generate separate files for each tilemap (keeping sub-module structure for LSP performance)
        {
            use std::fs;
            use std::path::Path;

            // Get the directory of the main file and the module name
            let main_path = Path::new(&full_path);
            let output_dir =
                main_path.parent().expect("Could not get parent directory of output file");
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
                code.write_header(self.allow_unused, self.use_crate_assets, true);

                // Write the tilemap
                code.write_line(&format!(
                    "pub const MAP_{}: Tilemap<{}> = Tilemap {{",
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
                println!(
                    "cargo:warning=Formatting and writing tilemap file: {}",
                    map_file_path_str
                );
                code.format_output(map_file_path_str);
                println!("cargo:warning=Tilemap file write completed: {}", map_file_path_str);
            }
        }

        // Write Bank struct constant
        let bank_name = self.name.to_uppercase();
        if self.write_colors && self.write_tiles {
            code.write_line(&format!("pub const BANK_{}: Bank = Bank {{", bank_name));

            code.write_line(&format!("  colors: COLORS_{}, ", bank_name,));
            code.write_line(&format!("  tiles: TILES_{}, ", bank_name));

            code.write_line("};");
            code.write_line("");
        }

        // Write bank colors
        if self.write_colors {
            code.write_line(&format!(
                "pub const COLORS_{}: ColorBank = ColorBank::new_from(",
                bank_name
            ));
            code.write_line("    &[");
            for color in &self.palette.rgb_colors {
                code.write_line(&format!(
                    "        RGBA12::with_transparency({}, {}, {}, {}),",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                ));
            }
            code.write_line("    ],");
            code.write_line("    &[");
            for mapping in &self.color_mappings {
                let values: Vec<String> = mapping.iter().map(|v| v.to_string()).collect();
                code.write_line(&format!("        [{}],", values.join(", ")));
            }
            code.write_line("    ],");
            code.write_line(");");
            code.write_line("");
        }

        if self.write_tiles {
            // Write bank tiles
            code.write_line(&format!(
                "pub const TILES_{}: TileBank = TileBank::new_from(",
                bank_name
            ));
            code.write_line("    &[");
            for tile_pixels in self.pixels.chunks(TILE_LEN) {
                code.write_line(&format!("        {},", crate::format_tile_compact(tile_pixels)));
            }
            code.write_line("    ],");
            code.write_line(");");
            code.write_line("");
        }

        // Write animation strips if any
        if !self.strips.is_empty() && self.write_animations {
            for strip in self.strips.values() {
                code.write_line(&format!(
                    "pub const STRIP_{}: [TilemapRef; {}] = [",
                    strip.name.to_uppercase(),
                    strip.frames.len()
                ));

                for (i, frame) in strip.frames.iter().enumerate() {
                    code.write_line("    TilemapRef {");
                    code.write_line(&format!("        cells: &FRAMES_{}[{}],", strip.name, i));
                    code.write_line(&format!("        columns: {},", frame.columns));
                    code.write_line(&format!("        rows: {},", frame.rows));
                    code.write_line("    },");
                }

                code.write_line("];");
                code.write_line("");
            }

            for strip in self.strips.values() {
                code.write_line(&format!(
                    "pub const FRAMES_{}: [[Cell; {}]; {}] = [",
                    strip.name.to_uppercase(),
                    strip.frames[0].cells.len(),
                    strip.frames.len()
                ));
                for frame in &strip.frames {
                    code.write_line("           [");
                    for cell in &frame.cells {
                        code.write_cell(cell);
                    }
                    code.write_line("           ],");
                }
                code.write_line("        ];");
                code.write_line("");
            }
        }

        // Write animations
        if self.write_animations {
            for anim in &self.anims {
                let Some(_strip) = self.strips.get(&anim.strip_name) else {
                    panic!("Invalid strip name for Anim: {}", anim.name)
                };
                code.write_line(&format!(
                    "pub const ANIM_{}: Anim = Anim {{",
                    anim.name.to_uppercase(),
                ));
                code.write_line(&format!("   fps: {},", anim.fps));
                code.write_line(&format!("   repeat: {},", anim.repeat));
                code.write_line(&format!("   frames: &{:?},", anim.frames.as_slice()));
                code.write_line(&format!(
                    "   strip: &STRIP_{}",
                    anim.strip_name.to_ascii_uppercase()
                ));
                code.write_line("        };");
                code.write_line("");
            }
        }

        // Write single tiles
        if self.write_tiles {
            for tile in &self.single_tiles {
                code.write_line(&format!(
                    "pub const {}: Cell = {};",
                    tile.name.to_uppercase(),
                    crate::format_cell_compact(&tile.cell)
                ));
            }
            if !self.single_tiles.is_empty() {
                code.write_line("");
            }
        }

        // Format the output
        println!("cargo:warning=Formatting and writing file: {}", full_path);
        code.format_output(&full_path);
        println!("cargo:warning=File write completed: {}", full_path);

        // Register this file for mod.rs generation only if it has content
        let has_content = !self.maps.is_empty()
            || !self.single_tiles.is_empty()
            || !self.anims.is_empty()
            || !self.strips.is_empty()
            || !self.pixels.is_empty()
            || !self.palette.rgb_colors.is_empty();

        if has_content {
            crate::register_generated_file(&full_path);
        }

        // Mark all input files as processed
        for command in &self.deferred_commands {
            let file_path = match command {
                DeferredCommand::NewEmptyTile | DeferredCommand::NewAnim { .. } => "",
                DeferredCommand::NewTile { path } => path,
                DeferredCommand::NewGroup { path, .. } => path,
                DeferredCommand::NewMap { path, .. } => path,
                DeferredCommand::NewAnimationStrip { path, .. } => path,
            };
            if !file_path.is_empty() {
                crate::mark_file_processed(file_path);
            }
        }
    }

    #[inline(always)]
    fn extract_tile_pixels(img: &PalettizedImg, abs_col: usize, abs_row: usize) -> Pixels {
        let mut tile_data = [0u8; TILE_LEN];
        for y in 0..TILE_SIZE as usize {
            for x in 0..TILE_SIZE as usize {
                let abs_x = (TILE_SIZE as usize * abs_col) + x;
                let abs_y = (TILE_SIZE as usize * abs_row) + y;
                let index = (img.width * abs_y) + abs_x;
                let color = img.pixels[index];
                tile_data[(TILE_SIZE as usize * y) + x] = color;
            }
        }
        tile_data
    }

    fn handle_groups(&mut self, canonical_tile: &CanonicalTile, group: Option<u8>) {
        if let Some(group_idx) = group {
            self.groups.register_tile(canonical_tile.pixels, group_idx);

            // Also register all transformations if enabled
            if self.allow_tile_transforms {
                for flip_x in [false, true] {
                    for flip_y in [false, true] {
                        for rotation in [false, true] {
                            if !flip_x && !flip_y && !rotation {
                                continue; // Skip identity transform
                            }

                            let transformed_tile = Self::transform_tile(
                                &canonical_tile.pixels,
                                flip_x,
                                flip_y,
                                rotation,
                            );

                            self.groups.register_tile(transformed_tile, group_idx);
                        }
                    }
                }
            }
        }
    }

    /// Main workhorse. Creates a tile map while storing tile pixels and subpalettes
    fn add_tiles(&mut self, img: &PalettizedImg, group: Option<u8>) -> Vec<Vec<Cell>> {
        let mut frames = vec![];

        // Iterate animation frames, then tiles within frames.
        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                let mut frame_tiles = vec![];

                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        // Absolute coordinates
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        // Extract pixels mapped to main palette (no mapping)
                        let source_pixels = Self::extract_tile_pixels(img, abs_col, abs_row);

                        // Redefine pixels as canonical pixels from this point on
                        let canonical_tile = self.create_canonical_tile(&source_pixels);

                        // Check if this tile (or any transformation) is already
                        // associated with a pre-generated Cell. Does not store tiles yet.
                        let mut found_cell = None;
                        if let Some(existing) = self.tiles_to_cells.get(&canonical_tile.pixels) {
                            // Define cell for this tile
                            found_cell = Some(*existing);
                        // If not, store canonical tile + associated cell
                        } else if self.allow_tile_transforms {
                            // Try all 7 other transformations using remapped data
                            'outer: for flip_x in [false, true] {
                                for flip_y in [false, true] {
                                    for rotation in [false, true] {
                                        if !flip_x && !flip_y && !rotation {
                                            continue;
                                        }

                                        let transformed_pixels = Self::transform_tile(
                                            &canonical_tile.pixels,
                                            flip_x,
                                            flip_y,
                                            rotation,
                                        );

                                        let transformed =
                                            self.create_canonical_tile(&transformed_pixels);

                                        if let Some(existing) =
                                            self.tiles_to_cells.get(&transformed.pixels)
                                        {
                                            found_cell = Some(*existing);
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        // If we're registering a group, store this canonical pattern (but skip empty tiles)
                        self.handle_groups(&canonical_tile, group);

                        // Look up group membership for this tile pattern
                        let group_bits =
                            self.groups.hash.get(&canonical_tile.pixels).copied().unwrap_or(0);

                        // Insert or reuse cell
                        let cell = match found_cell {
                            Some(existing_cell) => {
                                // Found existing tile with same pattern
                                // Get the canonical tile info for the existing tile
                                let color_mapping_idx = if let Some(existing_canonical) =
                                    self.canonical_tiles.get(&canonical_tile.pixels)
                                {
                                    // Clone the mapping to avoid borrow issues
                                    let existing_mapping = existing_canonical.mapping.clone();
                                    // Create remapping from current tile's colors to existing tile's colors
                                    self.create_color_remapping(
                                        &canonical_tile.mapping,
                                        &existing_mapping,
                                    )
                                } else {
                                    0 // Default identity mapping
                                };

                                // Safety check with useful error.
                                if color_mapping_idx > COLOR_MAPPING_COUNT {
                                    panic!(
                                        "\x1b[31mVideochip Error: \x1b[33mTile exceeds {} color mappings limit!\n\
                                        \tFrame: ({}, {})\n\
                                        \tTile within frame: row {}, col {}\n\
                                        \tAbsolute tile position: row {}, col {}\n\
                                        \tAttempted mapping {}\x1b[0m",
                                        COLOR_MAPPING_COUNT,
                                        frame_h,
                                        frame_v,
                                        row,
                                        col,
                                        abs_row,
                                        abs_col,
                                        color_mapping_idx
                                    );
                                }

                                // Define cell
                                Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                    group: group_bits,
                                    color_mapping: color_mapping_idx,
                                }
                            },
                            None => {
                                // Create new cell - new tiles always use identity mapping
                                let new_cell = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                    group: group_bits,
                                    color_mapping: 0, // Identity mapping for new tiles
                                };

                                // Store the canonical tile info
                                self.canonical_tiles
                                    .insert(canonical_tile.pixels.clone(), canonical_tile.clone());

                                // Store the actual pixel data
                                self.pixels.extend_from_slice(&source_pixels);

                                // Store remapped tile in hash (after remapping is complete)
                                // Store all transformations using remapped data
                                if self.allow_tile_transforms {
                                    for flip_x in [false, true] {
                                        for flip_y in [false, true] {
                                            for rotation in [false, true] {
                                                let transformed_pixels = Self::transform_tile(
                                                    &source_pixels,
                                                    flip_x,
                                                    flip_y,
                                                    rotation,
                                                );

                                                let transformed =
                                                    self.create_canonical_tile(&transformed_pixels);

                                                // Only store if this transformation produces different data
                                                if !self
                                                    .tiles_to_cells
                                                    .contains_key(&transformed.pixels)
                                                {
                                                    let mut cell_with_flags = new_cell;
                                                    cell_with_flags.flags.set_flip_x(flip_x);
                                                    cell_with_flags.flags.set_flip_y(flip_y);
                                                    cell_with_flags.flags.set_rotation(rotation);

                                                    self.tiles_to_cells.insert(
                                                        transformed.pixels.clone(),
                                                        cell_with_flags,
                                                    );

                                                    // Also store the canonical tile for the transformed pattern
                                                    self.canonical_tiles
                                                        .insert(transformed.pixels, transformed);
                                                }
                                            }
                                        }
                                    }
                                }
                                self.next_tile += 1;

                                new_cell
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

    fn load_valid_image(&mut self, path: &str, frames_h: u8, frames_v: u8) -> PalettizedImg {
        let img = PalettizedImg::from_image(path, frames_h, frames_v, self.palette);
        assert!(
            img.width % TILE_SIZE as usize == 0,
            "Single tile width must be multiple of {}",
            TILE_SIZE
        );
        img
    }

    /// A canonical tile stores the "structure" of a tile, not the actual colors, so that tiles with
    /// the same structure but different colors can still be detected as the same, but with different
    /// palettes.
    /// The mapping is like a mini-palette with each color assigned to a normalized index
    fn create_canonical_tile(&mut self, tile_pixels: &Pixels) -> CanonicalTile {
        // Create canonical representation by normalizing color indices
        let mut unique_colors = HashMap::new();
        let mut mapping = Vec::<u8>::new();
        let mut next_index = 0u8;

        // First pass: create canonical pixels with normalized indices
        let canonical_pixels: Pixels = std::array::from_fn(|i| {
            let source_color = tile_pixels[i];
            *unique_colors.entry(source_color).or_insert_with(|| {
                let canonical_color = next_index;
                mapping.push(source_color);
                next_index += 1;
                canonical_color
            })
        });

        // Create neighbor mask based on canonical pixels
        let mut pixels = [0u8; TILE_LEN];
        let size = TILE_SIZE as u32;
        for y in 0..size {
            for x in 0..size {
                let index = Self::get_index(x, y, size);
                pixels[index] = Self::neighbor_mask(&canonical_pixels, x, y, size, size)
            }
        }

        CanonicalTile { pixels, mapping }
    }

    /// Generates a copy of the tile pixels with some transformation
    fn transform_tile(tile: &Pixels, flip_x: bool, flip_y: bool, rotation: bool) -> Pixels {
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

    /// Returns a u8 mask where each bit is one if the neighbor at that position
    /// matches the desired value. Bit order is:
    /// top_left, top_middle, top_right,
    /// middle_left, middle_right,
    /// bottom_left, bottom_middle, bottom_right
    fn neighbor_mask<T>(map: &[T], x: u32, y: u32, width: u32, height: u32) -> u8
    where
        T: PartialEq,
    {
        let mut mask = 0;
        if x >= width || y >= height {
            return 0;
        }

        let desired_value = {
            let index = Self::get_index(x, y, width);
            &map[index]
        };

        let mut check = |dx: i32, dy: i32, value_to_add: u8| {
            let target_x = x as i32 + dx;
            let target_y = y as i32 + dy;
            if target_x >= 0 && target_y >= 0 {
                if (target_x as u32) < width && (target_y as u32) < height {
                    let index = Self::get_index(target_x as u32, target_y as u32, width);
                    let neighbor = &map[index];
                    if *desired_value == *neighbor {
                        mask += value_to_add;
                    }
                }
            }
        };

        // Checking all 8 neighbors left to right, top to bottom,
        // excluding center tile
        check(-1, -1, 128);
        check(0, -1, 64);
        check(1, -1, 32);
        check(-1, 0, 16);
        check(1, 0, 8);
        check(-1, 1, 4);
        check(0, 1, 2);
        check(1, 1, 1);
        mask
    }

    #[inline]
    fn get_index(x: u32, y: u32, map_width: u32) -> usize {
        (y as usize * map_width as usize) + x as usize
    }

    /// Create a color remapping from source tile colors to target tile colors
    fn create_color_remapping(&mut self, source_mapping: &[u8], target_mapping: &[u8]) -> u8 {
        let mut remapping = [0u8; COLORS_PER_PALETTE as usize];

        // Initialize with identity mapping
        for i in 0..COLORS_PER_PALETTE as usize {
            remapping[i] = i as u8;
        }

        // Build the actual remapping
        // source_mapping maps canonical indices to source colors
        // target_mapping maps canonical indices to target colors
        // We need to map target colors to source colors
        // (because we're using the target/stored tile's pixels but want source's colors)
        for (canonical_idx, &source_color) in source_mapping.iter().enumerate() {
            if canonical_idx < target_mapping.len() {
                let target_color = target_mapping[canonical_idx];
                // Map from stored tile's color to desired color
                remapping[target_color as usize] = source_color;
            }
        }

        // Check if this is the identity mapping
        let is_identity = remapping.iter().enumerate().all(|(i, &v)| i == v as usize);
        if is_identity {
            return 0; // Always use index 0 for identity mapping
        }

        // Check if this mapping already exists
        for (idx, existing_mapping) in self.color_mappings.iter().enumerate() {
            if *existing_mapping == remapping {
                return idx as u8;
            }
        }

        // Add new mapping
        let mapping_idx = self.color_mappings.len();
        println!("cargo:warning=Adding color mapping #{}: {:?}", mapping_idx, remapping);
        self.color_mappings.push(remapping);
        mapping_idx as u8
    }
}
