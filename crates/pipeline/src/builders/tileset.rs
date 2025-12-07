use tato_video::*;

use super::*;
use crate::*;
use std::collections::{HashMap, HashSet};

const TILE_LEN: usize = TILE_SIZE as usize * TILE_SIZE as usize;

// // Colors remapped to canonical form (0,1,2,3...) to allow detection of palette-swapped tiles!
// pub(crate) type CanonicalTile = [u8; TILE_LEN];

#[derive(Debug)]
pub(crate) struct CanonicalTile {
    pub pixels: [u8; TILE_LEN],
    pub mapping: Vec<u8>,
}

// Color mapped pixels in a tile
pub(crate) type Pixels = [u8; TILE_LEN];

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
    name: String,
    pixels: Vec<u8>,
    tiles_to_cells: HashMap<Pixels, Cell>,
    // stores the actual colored tile alongside the canonical tile, so that
    // we can obtain color mappings from it
    // original_color_mapping: HashMap<CanonicalTile, TilePixels>,
    palette: &'a mut PaletteBuilder,
    groups: &'a mut GroupBuilder,
    anims: Vec<AnimBuilder>,
    maps: Vec<MapBuilder>,
    single_tiles: Vec<SingleTileBuilder>,
    next_tile: u8,
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
            tiles_to_cells: HashMap::new(),
            // original_color_mapping: HashMap::new(),
            palette,
            groups,
            next_tile: 0,
            anims: vec![],
            maps: vec![],
            single_tiles: vec![],
            deferred_commands: Vec::new(),
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
        let mut should_regenerate = false;
        for command in &self.deferred_commands {
            // Check if any deferred command file needs regeneration
            let file_path = match command {
                DeferredCommand::NewGroup { path, .. } => path,
                DeferredCommand::NewTile { path } => path,
                DeferredCommand::NewMap { path, .. } => path,
                DeferredCommand::NewAnimationStrip { path, .. } => path,
            };
            if !file_path.is_empty() && crate::should_regenerate_file(file_path) {
                should_regenerate = true;
                break;
            }
        }
        if !should_regenerate {
            return;
        }

        println!("cargo:warning=Regenerating tileset: {}", file_path);
        // Execute all deferred commands now
        {
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
        {
            use std::fs;
            use std::path::Path;

            // Get the directory of the main file and the module name
            let main_path = Path::new(file_path);
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
                println!(
                    "cargo:warning=Formatting and writing tilemap file: {}",
                    map_file_path_str
                );
                code.format_output(map_file_path_str);
                println!("cargo:warning=Tilemap file write completed: {}", map_file_path_str);
            }
        }

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

        // if self.save_colors && !self.sub_palettes.is_empty() {
        //     code.write_line(&format!("    sub_palettes: Some(&["));
        //     for i in 0..self.sub_palettes.data().len() {
        //         code.write_line(&format!(
        //             "        &{}_SUBPALETTE_{},",
        //             self.name.to_uppercase(),
        //             i
        //         ));
        //     }
        //     code.write_line("    ]),");
        // } else {
        //     code.write_line("    sub_palettes: None,");
        // }

        code.write_line("};");
        code.write_line("");

        // Write palette colors
        if self.save_colors {
            code.write_color_array(&self.name, &self.palette.rgb_colors);
        }

        // // Write sub-palettes
        // if self.save_colors {
        //     for (i, sub_palette) in self.sub_palettes.data().iter().enumerate() {
        //         code.write_line(&format!(
        //             "#[unsafe(link_section = \"{}\")]",
        //             crate::get_platform_link_section()
        //         ));
        //         code.write_line(&format!(
        //             "pub static {}_SUBPALETTE_{}: [u8; {}] = [",
        //             self.name.to_uppercase(),
        //             i,
        //             // sub_palette.colors().len()
        //             COLORS_PER_TILE
        //         ));

        //         for n in 0..COLORS_PER_TILE as usize {
        //             let color_index = sub_palette.colors().get(n).unwrap_or(&0);
        //             code.write_line(&format!("    {},", color_index));
        //         }
        //         // for &color_index in sub_palette.colors() {
        //         //     code.write_line(&format!("    {},", color_index));
        //         // }

        //         code.write_line("];");
        //         code.write_line("");
        //     }
        // }

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
                "pub static {}_TILES: [Tile<4>; {}] = [",
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

    // fn handle_groups(&mut self, canonical_tile: CanonicalTile, group: Option<u8>) {
    //     if let Some(group_idx) = group {
    //         // // Only register multi-color tiles in groups (skip empty/solid-color tiles)
    //         // if color_mapping.len() > 1 {
    //         self.groups.register_tile(canonical_tile, group_idx);

    //         // Also register all transformations if enabled
    //         if self.allow_tile_transforms {
    //             for flip_x in [false, true] {
    //                 for flip_y in [false, true] {
    //                     for rotation in [false, true] {
    //                         if !flip_x && !flip_y && !rotation {
    //                             continue; // Skip identity transform
    //                         }

    //                         let transformed_tile =
    //                             Self::transform_tile(&canonical_tile, flip_x, flip_y, rotation);

    //                         self.groups.register_tile(transformed_tile, group_idx);
    //                     }
    //                 }
    //             }
    //         }
    //         // }
    //     }
    // }

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

                        // Palette handling
                        // let mut used_colors = HashSet::new();
                        // let mut mapping: HashMap<u8, usize> = HashMap::new();
                        // for color in source_pixels {
                        //     if !used_colors.contains(&color) {
                        //         mapping.insert(color, used_colors.len());
                        //         used_colors.insert(color);
                        //     }
                        // }

                        // // Safety check with useful error.
                        // let colors: Vec<u8> = unique_colors.into_iter().collect();
                        // if colors.len() > COLORS_PER_TILE as usize {
                        //     panic!(
                        //         "\x1b[31mVideochip Error: \x1b[33mTile exceeds {} color limit!\n\
                        //         \tFrame: ({}, {})\n\
                        //         \tTile within frame: row {}, col {}\n\
                        //         \tAbsolute tile position: row {}, col {}\n\
                        //         \tFound {} unique colors\x1b[0m",
                        //         COLORS_PER_TILE,
                        //         frame_h,
                        //         frame_v,
                        //         row,
                        //         col,
                        //         abs_row,
                        //         abs_col,
                        //         colors.len()
                        //     );
                        // }

                        // Redefine pixels as canonical pixels from this point on
                        let canonical_tile = self.create_canonical_tile(&source_pixels);

                        // Check if this tile (or any transformation) is already
                        // associated with a pre-generated Cell. Does not store tiles yet.
                        // TODO: Rearrange so for two paths, one with and one without
                        // tile transforms (no checking for flip_x, flip_y etc within the outer block)
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

                                        // TODO: Determine if this is right. I don't think I need to
                                        // regenerate the canonical tiles - just transform the existing
                                        // canonical tile sounds more right!
                                        let transformed =
                                            self.create_canonical_tile(&transformed_pixels);

                                        if let Some(existing) =
                                            self.tiles_to_cells.get(&transformed.pixels)
                                        {
                                            found_cell = Some(*existing);
                                            // found_cell = Some(Cell {
                                            //     id: existing.id,
                                            //     flags: existing
                                            //         .flags
                                            //         .with_horizontal_state(flip_x)
                                            //         .with_vertical_state(flip_y)
                                            //         .with_rotation_state(rotation),
                                            //     sub_palette: existing.sub_palette,
                                            //     group: existing.group,
                                            // });
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }

                        // Remap colors
                        // let subp = SubPalette::from(&colors);
                        // let subp_insert = self.sub_palettes.add(subp);

                        // let mapped_pixels: TilePixels = std::array::from_fn(|i| {
                        //     let source_color = source_pixels[i];
                        //     subp_insert.mapping[&source_color]
                        // });

                        // If we're registering a group, store this canonical pattern (but skip empty tiles)
                        // self.handle_groups(tile_pixels, group);

                        // Look up group membership for this tile pattern
                        // TODO: POSSIBLE BUG:
                        // May not be detecting transformed tiles. Will deal later.
                        let group_bits =
                            self.groups.hash.get(&canonical_tile.pixels).copied().unwrap_or(0);

                        // Insert or reuse cell
                        let cell = match found_cell {
                            Some(existing_cell) => {
                                // Found existing tile with same pattern
                                // Use the same sub-palette mapping we used for lookup
                                Cell {
                                    id: existing_cell.id,
                                    flags: existing_cell.flags,
                                    group: group_bits,
                                    color_mapping: 0, //TODO: implement color mapping
                                                      // sub_palette: existing_cell.sub_palette
                                }
                            },
                            None => {
                                // Create new cell using the sub-palette we already found/created
                                let new_cell = Cell {
                                    id: TileID(self.next_tile),
                                    flags: TileFlags::default(),
                                    group: group_bits,
                                    color_mapping: 0, //TODO: implement color mapping
                                };

                                // Store the already computed normalized_tile tile data
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

                                                // TODO: Determine if this is right. I don't think I need to
                                                // regenerate the canonical tiles - just transform the existing
                                                // canonical tile sounds more right!
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
                                                        transformed.pixels,
                                                        cell_with_flags,
                                                    );
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
        // Normalize indices for canonical representation
        // let mut unique_colors = HashMap::new(); //source color, canonical color
        // let mut mapping = Vec::<u8>::new();
        // let mut next_index = 0u8;
        // let canonical_pixels: Pixels = std::array::from_fn(|i| {
        //     let source_color = tile_pixels[i];
        //     *unique_colors.entry(source_color).or_insert_with(|| {
        //         let canonical_color = next_index;
        //         next_index += 1;
        //         mapping.push(source_color);
        //         canonical_color
        //     })
        // });
        //
        let mapping = Vec::new(); // not doing anything

        let mut pixels = [0u8; TILE_LEN];
        let size = TILE_SIZE as u32;
        for y in 0..size {
            for x in 0..size {
                let index = Self::get_index(x, y, size);
                // pixels[index] = Self::neighbor_mask(&canonical_pixels, x, y, size, size)
                pixels[index] = Self::neighbor_mask(tile_pixels, x, y, size, size)
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
}
