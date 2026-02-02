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
}

// Allows a single pass executing commands when writing
#[derive(Clone)]
enum DeferredCommand {
    NewEmptyTile,
    NewTile { path: String },
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
    // Store original source pixels by tile ID (for color remapping with transforms)
    original_source_pixels: HashMap<u8, Pixels>,
    palette: &'a mut PaletteBuilder,
    strips: HashMap<String, StripBuilder>,
    anims: Vec<Anim>,
    maps: Vec<MapBuilder>,
    single_tiles: Vec<SingleTileBuilder>,
    next_tile: u8,
    deferred_commands: Vec<DeferredCommand>,
}

impl<'a> BankBuilder<'a> {
    /// Creates a new bank builder with the given name, palette, and group references.
    pub fn new(name: &str, palette: &'a mut PaletteBuilder) -> Self {
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
            original_source_pixels: HashMap::new(),
            palette,
            strips: HashMap::new(),
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            next_tile: 0,
            deferred_commands: vec![],
        }
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
        struct _DeferredCommands;
        {
            let commands = self.deferred_commands.clone();
            for command in commands {
                match command {
                    DeferredCommand::NewEmptyTile => {
                        let img = PalettizedImg::empty(self.palette);
                        let cells = self.add_tiles(&img);
                        assert!(cells.len() == 1 && cells[0].len() == 1);
                        let tile_name = "EMPTY".to_string();
                        let single_tile =
                            SingleTileBuilder { name: tile_name, cell: cells[0][0].clone() };
                        self.single_tiles.push(single_tile);
                    },
                    DeferredCommand::NewTile { path } => {
                        let full_path = std::path::Path::new(&settings.asset_import_path)
                            .join(path)
                            .to_str()
                            .expect("Could not convert path to string")
                            .to_string();
                        let img = self.load_valid_image(&full_path, 1, 1);
                        assert!(
                            img.width == TILE_SIZE as usize,
                            "Single tile width must be {}",
                            TILE_SIZE
                        );
                        assert!(
                            img.cols_per_frame == 1 && img.rows_per_frame == 1,
                            "Single tile must be 1x1 tile (8x8 pixels)"
                        );
                        let cells = self.add_tiles(&img);
                        assert!(cells.len() == 1 && cells[0].len() == 1);
                        let tile_name = crate::strip_path_name(&full_path);
                        let single_tile =
                            SingleTileBuilder { name: tile_name, cell: cells[0][0].clone() };
                        self.single_tiles.push(single_tile);
                    },
                    DeferredCommand::NewMap { path, name } => {
                        let full_path = std::path::Path::new(&settings.asset_import_path)
                            .join(path)
                            .to_str()
                            .expect("Could not convert path to string")
                            .to_string();
                        let img = self.load_valid_image(&full_path, 1, 1);
                        let frames = self.add_tiles(&img);
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
                        let full_path = std::path::Path::new(&settings.asset_import_path)
                            .join(path)
                            .to_str()
                            .expect("Could not convert path to string")
                            .to_string();
                        let img = self.load_valid_image(&full_path, frames_h, frames_v);
                        let cells = self.add_tiles(&img);
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
                    map_file_path.to_str().expect("cargo:warning=Could not convert path to string");

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
            // code.write_line("    &[");
            // Color mappings are no longer used - colors are now per-cell via Palette
            // code.write_line("        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],");
            // code.write_line("    ],");
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

    /// Main workhorse. Creates a tile map while storing tile pixels and deduplicating.
    fn add_tiles(&mut self, img: &PalettizedImg) -> Vec<Vec<Cell>> {
        let mut frames = vec![];

        for frame_v in 0..img.frames_v as usize {
            for frame_h in 0..img.frames_h as usize {
                let mut frame_tiles = vec![];

                for row in 0..img.rows_per_frame as usize {
                    for col in 0..img.cols_per_frame as usize {
                        let abs_col = (frame_h * img.cols_per_frame as usize) + col;
                        let abs_row = (frame_v * img.rows_per_frame as usize) + row;

                        let source_pixels = Self::extract_tile_pixels(img, abs_col, abs_row);
                        let (canonical_tile, canonical_indices, palette_mapping) =
                            self.create_canonical_tile(&source_pixels);

                        // Try to find existing tile (direct match or via transformation)
                        let found_cell = self.find_matching_tile(&canonical_tile, &source_pixels);

                        let cell = match found_cell {
                            Some(existing_cell) => self.create_reused_cell(
                                existing_cell,
                                &source_pixels,
                                frame_h,
                                frame_v,
                                row,
                                col,
                                abs_row,
                                abs_col,
                            ),
                            None => self.create_new_tile(
                                canonical_tile,
                                canonical_indices,
                                palette_mapping,
                                source_pixels,
                                frame_h,
                                frame_v,
                                row,
                                col,
                                abs_row,
                                abs_col,
                            ),
                        };

                        frame_tiles.push(cell);
                    }
                }

                frames.push(frame_tiles);
            }
        }

        frames
    }

    /// Searches for an existing tile that matches (directly or via transformation).
    fn find_matching_tile(
        &mut self,
        canonical_tile: &CanonicalTile,
        source_pixels: &Pixels,
    ) -> Option<Cell> {
        // Check for direct match
        if let Some(existing) = self.tiles_to_cells.get(&canonical_tile.pixels) {
            return Some(*existing);
        }

        // Check transformed versions if enabled
        if self.allow_tile_transforms {
            for flip_x in [false, true] {
                for flip_y in [false, true] {
                    for rotation in [false, true] {
                        if !flip_x && !flip_y && !rotation {
                            continue; // Skip identity
                        }

                        let transformed_pixels =
                            Self::transform_tile(source_pixels, flip_x, flip_y, rotation);
                        let (transformed_canonical, _, _) =
                            self.create_canonical_tile(&transformed_pixels);

                        if let Some(existing) =
                            self.tiles_to_cells.get(&transformed_canonical.pixels)
                        {
                            return Some(*existing);
                        }
                    }
                }
            }
        }

        None
    }

    /// Creates a cell that reuses an existing tile with computed Palette.
    fn create_reused_cell(
        &mut self,
        existing_cell: Cell,
        source_pixels: &Pixels,
        frame_h: usize,
        frame_v: usize,
        row: usize,
        col: usize,
        abs_row: usize,
        abs_col: usize,
    ) -> Cell {
        let stored_pixels = self.original_source_pixels.get(&existing_cell.id.0).cloned().unwrap();
        let (_, stored_canonical, _) = self.create_canonical_tile(&stored_pixels);

        let tile_colors = self.compute_tile_colors_with_transform(
            source_pixels,
            &stored_canonical,
            existing_cell.flags.is_flipped_x(),
            existing_cell.flags.is_flipped_y(),
            existing_cell.flags.is_rotated(),
            frame_h,
            frame_v,
            row,
            col,
            abs_row,
            abs_col,
        );

        Cell {
            id: existing_cell.id,
            flags: existing_cell.flags,
            colors: tile_colors,
        }
    }

    /// Creates a new tile and stores all its transformed variants.
    fn create_new_tile(
        &mut self,
        canonical_tile: CanonicalTile,
        canonical_indices: Pixels,
        palette_mapping: Vec<u8>,
        source_pixels: Pixels,
        frame_h: usize,
        frame_v: usize,
        row: usize,
        col: usize,
        abs_row: usize,
        abs_col: usize,
    ) -> Cell {
        let tile_colors = self.create_tile_colors_from_mapping(
            &palette_mapping,
            frame_h,
            frame_v,
            row,
            col,
            abs_row,
            abs_col,
        );

        let new_cell = Cell {
            id: TileID(self.next_tile),
            flags: TileFlags::default(),
            colors: tile_colors,
        };

        // Store tile data
        self.canonical_tiles.insert(canonical_tile.pixels.clone(), canonical_tile);
        self.original_source_pixels.insert(self.next_tile, source_pixels);
        self.pixels.extend_from_slice(&canonical_indices);

        // Store all transformed variants for future matching
        self.store_transformed_variants(new_cell, &source_pixels);

        self.next_tile += 1;
        new_cell
    }

    /// Stores all transformed variants of a tile for deduplication.
    fn store_transformed_variants(&mut self, base_cell: Cell, source_pixels: &Pixels) {
        if !self.allow_tile_transforms {
            return;
        }

        for flip_x in [false, true] {
            for flip_y in [false, true] {
                for rotation in [false, true] {
                    let transformed_pixels =
                        Self::transform_tile(source_pixels, flip_x, flip_y, rotation);
                    let (transformed_canonical, _, _) =
                        self.create_canonical_tile(&transformed_pixels);

                    if !self.tiles_to_cells.contains_key(&transformed_canonical.pixels) {
                        let mut cell_with_flags = base_cell;
                        cell_with_flags.flags.set_flip_x(flip_x);
                        cell_with_flags.flags.set_flip_y(flip_y);
                        cell_with_flags.flags.set_rotation(rotation);

                        self.tiles_to_cells
                            .insert(transformed_canonical.pixels.clone(), cell_with_flags);
                        self.canonical_tiles
                            .insert(transformed_canonical.pixels.clone(), transformed_canonical);
                    }
                }
            }
        }
    }

    /// Creates Palette from a canonical palette mapping.
    /// The mapping vector contains palette colors in order of canonical indices:
    /// mapping[0] = palette color for canonical index 0
    /// mapping[1] = palette color for canonical index 1, etc.
    fn create_tile_colors_from_mapping(
        &self,
        palette_mapping: &[u8],
        frame_h: usize,
        frame_v: usize,
        row: usize,
        col: usize,
        abs_row: usize,
        abs_col: usize,
    ) -> Palette {
        if palette_mapping.len() > 4 {
            panic!(
                "\x1b[31mVideochip Error: \x1b[33mTile exceeds 4 color limit!\n\
                \tFrame: ({}, {})\n\
                \tTile within frame: row {}, col {}\n\
                \tAbsolute tile position: row {}, col {}\n\
                \tFound {} unique colors (max is 4)\n\
                \tColors: {:?}\x1b[0m",
                frame_h,
                frame_v,
                row,
                col,
                abs_row,
                abs_col,
                palette_mapping.len(),
                palette_mapping
            );
        }

        // The mapping already has palette colors in canonical index order
        let mut slot_colors = [0u8; 4];
        for (canonical_idx, &palette_color) in palette_mapping.iter().enumerate() {
            slot_colors[canonical_idx] = palette_color;
        }

        // Fill unused slots with 0 (transparent/default color)
        for i in palette_mapping.len()..4 {
            slot_colors[i] = 0;
        }

        Palette::new(slot_colors[0], slot_colors[1], slot_colors[2], slot_colors[3])
    }

    /// Computes Palette by mapping stored canonical indices to new palette colors,
    /// accounting for the rendering transformation.
    fn compute_tile_colors_with_transform(
        &self,
        new_pixels: &Pixels,
        stored_canonical: &Pixels,
        flip_x: bool,
        flip_y: bool,
        rotation: bool,
        frame_h: usize,
        frame_v: usize,
        row: usize,
        col: usize,
        abs_row: usize,
        abs_col: usize,
    ) -> Palette {
        let flags = TileFlags::default()
            .with_horizontal_state(flip_x)
            .with_vertical_state(flip_y)
            .with_rotation_state(rotation);

        let mut canonical_colors: [Option<u8>; 4] = [None; 4];
        let size = TILE_SIZE as usize;

        // Map canonical indices to palette colors by simulating rendering:
        // For each screen position, find which stored canonical index will be read,
        // then map it to the palette color the new tile wants at that screen position.
        for screen_y in 0..size {
            for screen_x in 0..size {
                let screen_idx = screen_y * size + screen_x;
                let (tile_x, tile_y) =
                    flags.transform_coords(screen_x as u8, screen_y as u8, TILE_SIZE);
                let tile_idx = tile_y as usize * size + tile_x as usize;

                let stored_canonical_idx = stored_canonical[tile_idx] as usize;
                let new_palette_color = new_pixels[screen_idx];

                if let Some(existing) = canonical_colors[stored_canonical_idx] {
                    if existing != new_palette_color {
                        // Inconsistency - shouldn't happen with proper matching
                    }
                } else {
                    canonical_colors[stored_canonical_idx] = Some(new_palette_color);
                }
            }
        }

        let mut canonical_to_palette = [0u8; 4];
        for i in 0..4 {
            canonical_to_palette[i] = canonical_colors[i].unwrap_or(0);
        }

        self.verify_color_limit(
            &canonical_to_palette,
            frame_h,
            frame_v,
            row,
            col,
            abs_row,
            abs_col,
        );

        Palette::new(
            canonical_to_palette[0],
            canonical_to_palette[1],
            canonical_to_palette[2],
            canonical_to_palette[3],
        )
    }

    /// Verifies that a tile doesn't exceed the 4 color limit.
    fn verify_color_limit(
        &self,
        colors: &[u8; 4],
        frame_h: usize,
        frame_v: usize,
        row: usize,
        col: usize,
        abs_row: usize,
        abs_col: usize,
    ) {
        let unique_count = colors.iter().collect::<std::collections::HashSet<_>>().len();
        if unique_count > 4 {
            panic!(
                "\x1b[31mVideochip Error: \x1b[33mTile exceeds 4 color limit!\n\
                \tFrame: ({}, {})\n\
                \tTile within frame: row {}, col {}\n\
                \tAbsolute tile position: row {}, col {}\n\
                \tFound {} unique colors (max is 4)\x1b[0m",
                frame_h, frame_v, row, col, abs_row, abs_col, unique_count
            );
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

    /// Creates a canonical tile representation for pattern matching and storage.
    /// Returns: (structure for matching, sorted indices for storage, color mapping).
    fn create_canonical_tile(&mut self, tile_pixels: &Pixels) -> (CanonicalTile, Pixels, Vec<u8>) {
        // Collect and sort unique colors for consistent canonical indices
        let mut sorted_colors: Vec<u8> = tile_pixels
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        sorted_colors.sort();

        // Map palette colors to canonical indices (0-3)
        let color_to_canonical: HashMap<u8, u8> =
            sorted_colors.iter().enumerate().map(|(idx, &color)| (color, idx as u8)).collect();

        // Convert tile pixels to canonical indices
        let canonical_pixels: Pixels = std::array::from_fn(|i| color_to_canonical[&tile_pixels[i]]);

        // Create structure representation using neighbor masks for pattern matching
        let structure_pixels = self.create_structure_mask(&canonical_pixels);

        (CanonicalTile { pixels: structure_pixels }, canonical_pixels, sorted_colors)
    }

    /// Creates a neighbor mask representation for structure-based tile matching.
    fn create_structure_mask(&self, canonical_pixels: &Pixels) -> [u8; TILE_LEN] {
        let mut structure_pixels = [0u8; TILE_LEN];
        let size = TILE_SIZE as u32;
        for y in 0..size {
            for x in 0..size {
                let index = Self::get_index(x, y, size);
                structure_pixels[index] = Self::neighbor_mask(canonical_pixels, x, y, size, size);
            }
        }
        structure_pixels
    }

    /// Applies transformation to tile pixels (flip_x, flip_y, rotation).
    fn transform_tile(tile: &Pixels, flip_x: bool, flip_y: bool, rotation: bool) -> Pixels {
        let mut result = [0u8; TILE_LEN];
        let size = TILE_SIZE as usize;
        let flags = TileFlags::default()
            .with_horizontal_state(flip_x)
            .with_vertical_state(flip_y)
            .with_rotation_state(rotation);

        // For each destination position, find the source position using the same
        // transform the renderer uses
        for dst_y in 0..size {
            for dst_x in 0..size {
                let (src_x, src_y) =
                    flags.transform_coords(dst_x as u8, dst_y as u8, TILE_SIZE as u8);
                let src_idx = src_y as usize * size + src_x as usize;
                let dst_idx = dst_y * size + dst_x;
                result[dst_idx] = tile[src_idx];
            }
        }
        result
    }

    /// Returns a bitmask indicating which neighbors match the center pixel's value.
    /// Bits represent: top_left, top_center, top_right, middle_left, middle_right,
    /// bottom_left, bottom_center, bottom_right.
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

        // Check all 8 neighbors (excluding center)
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
}
