use crate::*;

pub const SUB_PALETTE_COUNT: usize = 16;
pub const SUB_PALETTE_COLOR_COUNT: usize = 4;

pub struct Pipeline {
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
        println!(
            "cargo:warning=Working Dir:{:?}",
            std::env::current_dir().ok().unwrap()
        );
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=assets/*.*");
        // Create with defaults
        Pipeline {
            palettes: Vec::new(),
            tilesets: Vec::new(),
            tileset_head: 0,
            palette_head: 0,
        }
    }

    // TODO: Limit tileset entries to 256

    /// Initializes an empty tileset, returns its ID
    pub fn new_palette(&mut self, name: impl Into<String>, capacity: u8) -> PaletteID {
        let id: u8 = self.palette_head;
        self.palette_head += 1;
        println!(
            "cargo:warning=Pipeline: initializing palette at index {}.",
            id
        );
        self.palettes
            .push(PaletteBuilder::new(name.into(), capacity.into(), id));
        PaletteID(id)
    }

    /// Initializes an empty tileset, returns its ID
    pub fn new_tileset(&mut self, name: impl Into<String>, palette_id: PaletteID) -> TilesetID {
        let id: u8 = self.tileset_head;
        self.tileset_head += 1;
        println!(
            "cargo:warning=Pipeline: initializing tileset at index {}.",
            id
        );
        self.tilesets
            .push(TilesetBuilder::new(name.into(), palette_id));
        TilesetID(id)
    }

    /// Initializes a new animation sequence from a .png file. It will:
    /// - "Palettize" the image, adding its colors to the palette if needed
    /// - Divide the animation into tiles, added to the tileset
    /// - Create a new const Anim in the output file that carries the necessary data.
    pub fn new_anim(
        &mut self,
        path: &str,
        fps: u8,
        frames_h: u8,
        frames_v: u8,
        tileset_id: TilesetID,
        // group_id: impl GroupEnum,
    ) {
        let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
            panic!("Invalid tileset id: {:?}", tileset_id);
        };

        let palette = self
            .palettes
            .get_mut(tileset.palette_id.0 as usize)
            .unwrap();
        let img = PalettizedImg::from_image(path, frames_h, frames_v, palette);
        let tiles = tileset.add_tiles(&img, palette); //, group, false);
        let tiles_per_frame = img.cols_per_frame as usize * img.rows_per_frame as usize;
        let frame_count = img.frames_h as usize * img.frames_v as usize;

        assert!(frame_count > 0);

        let anim = AnimBuilder {
            name: strip_path_name(path),
            // id: tileset.anims.len().try_into().unwrap(),
            fps,
            columns: u8::try_from(img.cols_per_frame).unwrap(),
            rows: u8::try_from(img.rows_per_frame).unwrap(),
            frames: (0..frame_count)
                .map(|i| {
                    let index = i * tiles_per_frame;
                    FrameBuilder {
                        tiles: tiles[index..index + tiles_per_frame].into(),
                    }
                })
                .collect(),
        };

        tileset.anims.push(anim);
    }

    pub fn disable_tile_transform_detection(&mut self, tileset_id: TilesetID) {
        if let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) {
            tileset.allow_tile_transforms = false;
        }
    }

    pub fn write_all_assets(&mut self, file_path: &str) {
        // The code writer is created, modified and dropped in this scope
        // which means the file is ready to be formatted next.
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            code.write_line("use tato::Anim;");
            code.write_line("");
            self.append_palettes(&mut code);
            self.append_sub_palettes(&mut code);
            self.append_anims(&mut code);
            self.append_pixels(&mut code);
        }
        self.format_output(file_path);
    }

    pub fn write_palettes(&mut self, file_path: &str) {
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            self.append_palettes(&mut code);
        }
        self.format_output(file_path);
    }

    pub fn write_tileset(&mut self, file_path: &str) {
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            self.append_sub_palettes(&mut code);
            self.append_anims(&mut code);
            self.append_pixels(&mut code);
        }
        self.format_output(file_path);
    }

    pub fn write_tileset_pixels(&mut self, file_path: &str) {
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            self.append_pixels(&mut code);
        }
        self.format_output(file_path);
    }

    pub fn write_tileset_sub_palettes(&mut self, file_path: &str) {
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            self.append_sub_palettes(&mut code);
        }
        self.format_output(file_path);
    }

    pub fn write_tileset_anims(&mut self, file_path: &str) {
        {
            let mut code = CodeWriter::new(file_path);
            self.append_header(&mut code);
            code.write_line("use tato::Anim;");
            code.write_line("");
            self.append_anims(&mut code);
        }
        self.format_output(file_path);
    }
}

// ****************************** Code Gen ******************************

impl Pipeline {
    fn append_header(&mut self, code: &mut CodeWriter) {
        // Header
        code.write_line("// Auto-generated code - do not edit manually");
        code.write_line("#![allow(unused)]");
        code.write_line("");
        code.write_line("");
    }

    fn append_palettes(&mut self, code: &mut CodeWriter) {
        // Palettes
        for palette in &self.palettes {
            code.write_line(&format!(
                "pub const {}: [Color9Bit; {}] = [",
                palette.name.to_uppercase(),
                palette.colors.len()
            ));
            code.indent();

            for color in &palette.colors {
                code.write_line(&format!(
                    "Color9Bit::new({}, {}, {}),",
                    color.r(),
                    color.g(),
                    color.b()
                ));
            }

            code.dedent();
            code.write_line("];");
            code.write_line("");
        }
    }

    fn append_anims(&mut self, code: &mut CodeWriter) {
        // Tilesets
        for tileset in &self.tilesets {
            // Anims
            for anim in &tileset.anims {
                // println!("Anim: {:#?}", anim);
                code.write_line(&format!(
                    "pub const {}: Anim<{}, {}> = Anim {{ fps: {}, cols_per_frame: {}, frames: [",
                    anim.name.to_uppercase(),
                    anim.frames.len(),
                    anim.columns as usize * anim.rows as usize,
                    anim.fps,
                    anim.columns
                ));
                code.indent();
                for frame in &anim.frames {
                    code.start_line("[");
                    for tile in &frame.tiles {
                        code.write(&format!("{}, ", tile.index.0));
                    }
                    code.finish_line("],\n");
                }
                code.dedent();
                code.write_line("]};");
                code.write_line("");
            }
        }
    }

    fn append_sub_palettes(&mut self, code: &mut CodeWriter) {
        // Tilesets
        for tileset in &self.tilesets {
            // Sub-Palettes
            for sub_plt in &tileset.sub_palettes {
                code.write_line(&format!(
                    "pub const {}: [u8; {}] = [",
                    tileset
                        .sub_palette_name_hash
                        .get(sub_plt)
                        .unwrap()
                        .to_uppercase(),
                    sub_plt.len()
                ));
                code.indent();
                for color_index in sub_plt {
                    code.write_line(&format!("{},", color_index));
                }
                code.dedent();
                code.write_line("];");
                code.write_line("");
            }
        }
    }

    fn append_pixels(&mut self, code: &mut CodeWriter) {
        // Write Pixels at the bottom of the file!
        for tileset in &self.tilesets {
            code.write_line(&format!(
                "pub const TILESET_{}: [u8; {}] = [",
                tileset.name.to_uppercase(),
                tileset.pixels.len()
            ));
            code.indent();
            for pixel in &tileset.pixels {
                code.write(&format!("{}, ", pixel));
            }
            code.dedent();
            code.write_line("];");
            code.write_line("");
        }
    }

    fn format_output(&mut self, file_path: &str) {
        // Format the output file with rustfmt
        let output = std::process::Command::new("rustfmt")
            .arg(file_path)
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    println!("cargo:warning=Failed to format generated code: {}", error);
                } else {
                    println!("cargo:warning=Successfully formatted generated code");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run rustfmt: {}", e);
                println!(
                    "cargo:warning=Make sure rustfmt is installed (rustup component add rustfmt)"
                );
            }
        }
    }
}

fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}
