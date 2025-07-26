use crate::*;
use tato_video::*;

pub const SUB_PALETTE_COUNT: usize = 16;
pub const SUB_PALETTE_COLOR_COUNT: usize = 4;

pub struct Pipeline {
    /// Adds "allow unused" lint attribute to the top of the exported file
    pub allow_unused: bool,
    /// Generally should never be true, only used by the tato library for default assets
    pub use_crate_assets: bool,
    /// Allows skipping the palette when saving a tileset.
    /// Useful for assets intended to use the default palette only.
    pub save_palettes: bool,

    palettes: Vec<PaletteBuilder>,
    tilesets: Vec<TilesetBuilder>,
    tileset_head: u8,
    palette_head: u8,
}

impl Pipeline {
    /// Crates a new pipeline with no palettes or tilesets yet.
    pub fn new() -> Self {
        // Cargo build setup
        println!("cargo:warning=Running Build Script!");
        println!("cargo:warning=Working Dir:{:?}", std::env::current_dir().ok().unwrap());

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=assets/*.*");

        // Create with defaults
        Pipeline {
            allow_unused: false,
            use_crate_assets: false,
            save_palettes: true,
            palettes: Vec::new(),
            tilesets: Vec::new(),
            tileset_head: 0,
            palette_head: 0,
        }
    }

    // TODO: Limit tileset entries to 256

    /// Initializes an empty tileset, returns its ID
    pub fn new_palette(&mut self, name: impl Into<String>) -> PaletteID {
        let id: u8 = self.palette_head;
        self.palette_head += 1;
        println!("cargo:warning=Pipeline: initializing palette at index {}.", id);
        self.palettes.push(PaletteBuilder::new(name.into(), id));
        PaletteID(id)
    }

    /// Initializes an empty tileset, returns its ID
    pub fn new_tileset(
        &mut self,
        name: impl Into<String>,
        palette_id: PaletteID,
    ) -> TilesetBuilderID {
        let id: u8 = self.tileset_head;
        self.tileset_head += 1;
        println!("cargo:warning=Pipeline: initializing tileset at index {}.", id);
        self.tilesets.push(TilesetBuilder::new(name.into(), palette_id));
        TilesetBuilderID(id)
    }

    /// Initializes a new single tile from a .png file:
    pub fn new_tile(
        &mut self,
        path: &str,
        tileset_id: TilesetBuilderID,
        // group_id: impl GroupEnum,
    ) {
        let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
            panic!("Invalid tileset id: {:?}", tileset_id);
        };

        let palette = self.palettes.get_mut(tileset.palette_id.0 as usize).unwrap();
        let img = PalettizedImg::from_image(path, 1, 1, palette);
        assert!(img.width == TILE_SIZE as usize, "Single tile width must be {}", TILE_SIZE);

        let frames = tileset.add_tiles(&img, palette); //, group, false);

        tileset
            .single_tiles
            .push(SingleTileBuilder { name: strip_path_name(path), cell: frames[0][0] });
    }

    /// Initializes a new animation sequence from a .png file. It will:
    /// - "Palettize" the image, adding its colors to the palette if needed
    /// - Divide the animation into tiles, added to the tileset
    /// - Create a new const Anim in the output file that carries the necessary data.
    pub fn new_animation_strip(
        &mut self,
        path: &str,
        name: &str,
        // fps: u8,
        frames_h: u8,
        frames_v: u8,
        tileset_id: TilesetBuilderID,
    ) {
        let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
            panic!("Invalid tileset id: {:?}", tileset_id);
        };

        let palette = self.palettes.get_mut(tileset.palette_id.0 as usize).unwrap();
        let img = PalettizedImg::from_image(path, frames_h, frames_v, palette);
        let cells = tileset.add_tiles(&img, palette);
        let frame_count = img.frames_h as usize * img.frames_v as usize;

        assert!(frame_count > 0);
        // let id = tileset.anims.len();
        let anim = AnimBuilder {
            name: String::from(name),
            // fps,
            frames: (0..frame_count)
                .map(|i| MapBuilder {
                    name: format!("frame_{:02}", i),
                    columns: u8::try_from(img.cols_per_frame).unwrap(),
                    rows: u8::try_from(img.rows_per_frame).unwrap(),
                    cells: cells[i].clone(),
                })
                .collect(),
            tags: vec![],
        };

        tileset.anims.push(anim);
        // AnimBuilderID(id)
    }

    // pub fn new_anim(
    //     &mut self,
    //     tileset_id: TilesetBuilderID,
    //     anim_id: AnimBuilderID,
    //     name: &str,
    //     fps: u8,
    //     frames: &[u8],
    // ) {
    //     let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
    //         panic!("Invalid Tileset ID: {:?}", tileset_id);
    //     };
    //     let Some(anim) = &mut tileset.anims.get_mut(anim_id.0) else {
    //         panic!("Invalid AnimBuilder ID: {:?}", tileset_id);
    //     };
    //     let duration = ((1.0 / fps as f32) * 1000.0) as u16;
    //     let tag = AnimTag {
    //         name: String::from(name),
    //         steps: frames.iter().map(|i| FrameStep { index: *i, duration }).collect(),
    //     };
    //     anim.tags.push(tag)
    // }

    pub fn new_map(&mut self, path: &str, name: &str, tileset_id: TilesetBuilderID) {
        let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
            panic!("Invalid tileset id: {:?}", tileset_id);
        };

        let palette = self.palettes.get_mut(tileset.palette_id.0 as usize).unwrap();
        let img = PalettizedImg::from_image(path, 1, 1, palette);
        let cells = tileset.add_tiles(&img, palette); //, group, false);
        assert!(cells.len() == 1);

        let map = MapBuilder {
            name: String::from(name),
            columns: u8::try_from(img.cols_per_frame).unwrap(),
            rows: u8::try_from(img.rows_per_frame).unwrap(),
            cells: cells[0].clone(), // just the first frame, there's only 1 anyway!
        };

        tileset.maps.push(map);
    }

    // TODO: Pass as an argument? Or just automatically to this for "Font", when I have that
    pub fn disable_tile_transform_detection(&mut self, tileset_id: TilesetBuilderID) {
        if let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) {
            tileset.allow_tile_transforms = false;
        }
    }

    pub fn write_tileset(&mut self, tileset_id: TilesetBuilderID, file_path: &str) {
        // The code writer is created, modified and dropped in this scope
        // which means the file is ready to be formatted next.
        let mut code = CodeWriter::new(file_path);
        self.append_header(&mut code);
        // code.write_line("use tato::Anim;");
        code.write_line("");
        self.append_tileset_data(tileset_id, &mut code);
        self.append_palettes(tileset_id, &mut code);
        self.append_sub_palettes(tileset_id, &mut code);
        self.append_anims(tileset_id, &mut code);
        self.append_maps(tileset_id, &mut code);
        self.append_single_tile_ids(tileset_id, &mut code);
        self.append_tiles(tileset_id, &mut code);
        self.format_output(file_path);
    }

    pub fn write_palettes(&mut self, tileset_id: TilesetBuilderID, file_path: &str) {
        let mut code = CodeWriter::new(file_path);
        self.append_header(&mut code);
        self.append_palettes(tileset_id, &mut code);
        self.format_output(file_path);
    }

    pub fn write_pixels(&mut self, tileset_id: TilesetBuilderID, file_path: &str) {
        let mut code = CodeWriter::new(file_path);
        self.append_header(&mut code);
        self.append_single_tile_ids(tileset_id, &mut code);
        self.append_tiles(tileset_id, &mut code);
        self.format_output(file_path);
    }

    pub fn write_tileset_sub_palettes(&mut self, tileset_id: TilesetBuilderID, file_path: &str) {
        let mut code = CodeWriter::new(file_path);
        self.append_header(&mut code);
        self.append_sub_palettes(tileset_id, &mut code);
        self.format_output(file_path);
    }

    pub fn write_tileset_anims(&mut self, tileset_id: TilesetBuilderID, file_path: &str) {
        let mut code = CodeWriter::new(file_path);
        self.append_header(&mut code);
        code.write_line("use tato::Anim;");
        code.write_line("");
        self.append_anims(tileset_id, &mut code);
        self.format_output(file_path);
    }
}

// ****************************** Code Gen ******************************
impl Pipeline {
    fn append_header(&mut self, code: &mut CodeWriter) {
        // Header
        code.write_line("// Auto-generated code - do not edit manually");
        if self.allow_unused {
            code.write_line("#![allow(unused)]");
        }
        if self.use_crate_assets {
            code.write_line("use crate::prelude::*;");
        } else {
            code.write_line("use tato::prelude::*;");
        }
        code.write_line("");
        code.write_line("");
    }

    fn append_tileset_data(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        let palette = &mut self.palettes.get(tileset.palette_id.0 as usize).unwrap();

        code.write_line(&format!(
            "pub const {}_TILESET: TilesetData = TilesetData{{",
            palette.name.to_uppercase(),
        ));

        code.write_line(&format!("    tiles: &{}_TILES,", tileset.name.to_uppercase()));
        if self.palettes.len() > 0 && self.save_palettes {
            code.write_line(&format!("    colors: Some(&{}_COLORS),", tileset.name.to_uppercase()));
        } else {
            code.write_line(&format!("    colors: None,"));
        }

        // TODO: Should this also check for save_palettes? Not sure.
        if tileset.sub_palettes.len() > 0 && self.save_palettes {
            code.write_line(&format!("    sub_palettes: Some(&["));
            for (i, _sub_plt) in tileset.sub_palettes.iter().enumerate() {
                code.write_line(&format!("&{}_SUBPALETTE_{},", tileset.name.to_uppercase(), i,));
            }
            code.write_line("])");
        } else {
            code.write_line(&format!("    sub_palettes: None,"));
        }

        code.write_line("};");
        code.write_line("");
    }

    fn append_palettes(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        if !self.save_palettes {
            return;
        }
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        let palette = &mut self.palettes.get(tileset.palette_id.0 as usize).unwrap();
        if palette.colors.len() > 0 {
            code.write_line(&format!(
                "pub const {}_COLORS: [RGBA12; {}] = [",
                palette.name.to_uppercase(),
                palette.colors.len()
            ));

            for color in &palette.colors {
                code.write_line(&format!(
                    "RGBA12::new({}, {}, {}, {}),",
                    color.r(),
                    color.g(),
                    color.b(),
                    color.a()
                ));
            }

            code.write_line("];");
            code.write_line("");
        }
    }

    fn append_sub_palettes(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        if !self.save_palettes {
            return;
        }
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        // Sub-Palettes
        for sub_plt in &tileset.sub_palettes {
            let name_str = tileset
                .sub_palette_name_hash
                .get(sub_plt)
                .unwrap() //
                .to_uppercase();
            let mut name_parts = name_str.split('_');
            let left_name = name_parts.next().unwrap();
            let right_name = name_parts.next().unwrap_or("");
            code.write_line(&format!(
                "pub const {}_SUBPALETTE_{}: [u8; {}] = [",
                left_name,
                right_name,
                sub_plt.len()
            ));

            for color_index in sub_plt {
                code.write_line(&format!("{},", color_index));
            }

            code.write_line("];");
            code.write_line("");
        }
    }

    fn append_anims(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        // Tilesets
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        for anim in &tileset.anims {
            code.write_line(&format!(
                "pub const {}: [Tilemap<{}>; {}] = [",
                anim.name.to_uppercase(),
                anim.frames[0].cells.len(),
                anim.frames.len(),
            ));

            // Frames
            for frame in &anim.frames {
                code.write_line("Tilemap{");
                code.write_line("cells:[");

                // Write cells in rows
                for row in 0..frame.rows {
                    for col in 0..frame.columns {
                        let index = (row as usize * frame.columns as usize) + col as usize;
                        let cell = &frame.cells[index];
                        code.write_line(&format!("    {:?},", cell));
                    }
                    code.write_line("");
                }

                code.write_line("],");
                code.write_line(&format!("columns:{},", frame.columns));
                code.write_line(&format!("rows:{},", frame.rows));
                code.write_line("},");
            }
            code.write_line("];");

            // Anims
            for tag in &anim.tags {
                code.write_line(&format!(
                    "pub const {}:[FrameStep; {}] = [",
                    tag.name,
                    tag.steps.len(),
                ));
                for step in &tag.steps {
                    code.write("FrameStep::HoldMillisecs {");
                    code.write(&format!("frame: {},", step.index));
                    code.write(&format!("duration: {},", step.duration));
                    code.write("},");
                    code.write("");
                }
                code.write_line("];");
            }
            code.write_line("");
        }
    }

    fn append_maps(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        for map in &tileset.maps {
            code.write_line(&format!(
                "pub const {}: Tilemap<{}> = Tilemap {{",
                map.name.to_uppercase(),
                map.cells.len(),
            ));
            code.write_line(&format!("columns: {},", map.columns));
            code.write_line(&format!("rows: {},", map.rows));
            code.write_line(&format!("cells: [",));
            for cell in &map.cells {
                code.write_line(&format!("    {:?},", cell));
            }
            code.write_line("],");
            code.write_line("};");
            code.write_line("");
        }
    }

    fn append_single_tile_ids(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        for tile_builder in &tileset.single_tiles {
            code.write_line(&format!(
                "pub const {}: TileID = {:?};",
                tile_builder.name.to_uppercase(),
                tile_builder.cell.id,
            ));
        }
    }

    fn append_tiles(&mut self, tileset_id: TilesetBuilderID, code: &mut CodeWriter) {
        let tileset = &mut self.tilesets.get(tileset_id.0 as usize).unwrap();
        // Write Pixels at the bottom of the file!
        code.write_line("");
        code.write_line(&format!(
            "pub const {}_TILES: [Tile<2>; {}] = [",
            tileset.name.to_uppercase(),
            tileset.pixels.len() / 64
        ));

        for tile in tileset.pixels.chunks(64) {
            code.write_line("Tile {");
            code.write_line("clusters:[");

            for pixels in tile.chunks(8) {
                let cluster = Cluster::<2>::from(pixels);
                code.write(&format!("{:?},", cluster));
            }

            code.write_line("],");
            code.write_line("},");
        }

        code.write_line("];");
        code.write_line("");
    }

    fn format_output(&mut self, file_path: &str) {
        // Format the output file with rustfmt
        let output = std::process::Command::new("rustfmt").arg(file_path).output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to format generated code: {}", error);
                } else {
                    println!("cargo:warning=Successfully formatted generated code");
                }
            },
            Err(e) => {
                println!("cargo:warning=Failed to run rustfmt: {}", e);
                println!(
                    "cargo:warning=Make sure rustfmt is installed (rustup component add rustfmt)"
                );
            },
        }
    }
}

fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}
