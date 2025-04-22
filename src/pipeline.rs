use tato::video::{TileID, TILE_SIZE};

use crate::*;

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
    // TODO: Limit palette count to an arbitrary number

    /// Initializes an empty tileset, returns its ID
    pub fn new_palette(&mut self, name: impl Into<String>) -> PaletteID {
        let id: u8 = self.palette_head;
        self.palette_head += 1;
        println!(
            "cargo:warning=Pipeline: initializing palette at index {}.",
            id
        );
        self.palettes.push(PaletteBuilder::new(name.into(), 16, id));
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
        palette_id: PaletteID,
        // group_id: impl GroupEnum,
    ) {
        let Some(tileset) = self.tilesets.get_mut(tileset_id.0 as usize) else {
            panic!("Invalid tileset id: {:?}", tileset_id);
        };

        let palette_id = palette_id.0 as usize;
        let palette = self.palettes.get_mut(palette_id).unwrap();

        // let frame_layout = Some((frames_h, frames_v));
        let img = PalettizedImg::from_image(path, palette);
        println!("cargo:warning= image: {:?}", img);

        let tiles = tileset.add_tiles(&img); //, group, false);
        let tiles_h = img.width / TILE_SIZE as usize;
        let tiles_v = img.height / TILE_SIZE as usize;
        let cols_per_frame = tiles_h / frames_h as usize;
        let rows_per_frame = tiles_v / frames_v as usize;
        let tiles_per_frame = cols_per_frame * rows_per_frame;
        let frame_count = frames_h as usize * frames_v as usize;
        println!("cargo:warning= frame count: {}", frame_count);

        assert!(frame_count > 0);

        let anim = AnimBuilder {
            name: strip_path_name(path),
            // id: tileset.anims.len().try_into().unwrap(),
            fps,
            columns: u8::try_from(cols_per_frame).unwrap(),
            frames: (0..frame_count)
                .map(|n| {
                    let index = n * tiles_per_frame;
                    FrameBuilder::from_slice(
                        &tiles[index..index + tiles_per_frame],
                        // img.cols_per_frame,
                        // img.rows_per_frame,
                    )
                })
                .collect(),
        };

        // let anim = AnimBuilder {
        //     name: strip_path_name(path),
        //     fps,
        //     columns: u8::try_from(cols_per_frame).unwrap(),
        //     frames: (0..frame_count)
        //         // This filter removes any repeated index
        //         .filter_map(|n| {
        //             let index = n * tiles_per_frame;
        //             let frame_tiles = &tiles[index..index + tiles_per_frame];

        //             // Check if any tile in this frame has an index >= frames.len()
        //             let highest_index = frame_tiles.iter()
        //                 .map(|tile| tile.index)
        //                 .max()
        //                 .unwrap_or(TileID(0));

        //             if highest_index.0 >= n as u8 {
        //                 Some(FrameBuilder::from_slice(frame_tiles))
        //             } else {
        //                 None
        //             }
        //         })
        //         .collect(),
        // };

        println!("cargo:warning={:?}", anim);
        tileset.anims.push(anim);
    }

    // ****************************** Code Gen ******************************

    pub fn write_assets(&mut self, file_path: &str) {
        let mut code = CodeWriter::new(file_path);

        // Header
        code.write_line("// Auto-generated code - do not edit manually");
        code.write_line("#![allow(unused)]");
        code.write_line("");
        code.write_line("use tato::video::*;");
        code.write_line("use tato::Anim;");
        code.write_line("");

        // Palettes
        for palette in &self.palettes {
            code.write_line(&format!(
                "pub const {}: [Color9Bit; {}] = [",
                palette.name.to_uppercase(),
                palette.colors().len()
            ));
            code.indent();

            for color in palette.colors() {
                code.write_line(&format!(
                    "Color9Bit::new({},{},{}),",
                    color.r(),
                    color.g(),
                    color.b()
                ));
            }

            code.dedent();
            code.write_line("];");
            code.write_line("");
        }

        // Anims
        for tileset in &self.tilesets {
            for anim in &tileset.anims {
                code.write_line(&format!(
                    "pub const {}: Anim<{}, 1> = Anim {{ fps: {}, cols_per_frame: {}, frames: [",
                    anim.name.to_uppercase(),
                    anim.frames.len(),
                    anim.fps,
                    anim.columns
                ));
                code.indent();

                code.dedent();
                code.write_line("]};");
            }
        }
    }

    // Methods for tileset generation
    // pub fn generate_tileset(&mut self, name: &str, tileset: &TilesetBuilder) {
    //     // Generate pixel data
    //     code.write_line(&format!(
    //         "pub const {}_PIXELS: [u8; {}] = [",
    //         name.to_uppercase(),
    //         tileset.pixels.len()
    //     ));
    //     code.indent();

    //     // Format pixels in rows of 16 for readability
    //     for chunk in tileset.pixels.chunks(16) {
    //         let values = chunk
    //             .iter()
    //             .map(|p| format!("{}", p))
    //             .collect::<Vec<_>>()
    //             .join(", ");
    //         code.write_line(&format!("{},", values));
    //     }

    //     code.dedent();
    //     code.write_line("];");
    //     code.write_line("");

    //     // Generate tile definitions
    //     code.generate_tiles(name, tileset);

    //     // Generate animations
    //     code.generate_animations(name, tileset);

    //     // Generate fonts
    //     code.generate_fonts(name, tileset);

    //     // Generate tilemaps
    //     code.generate_tilemaps(name, tileset);
    // }

    // ... more methods for specific asset types
}

fn strip_path_name(path: &str) -> String {
    let split = path.split('/');
    let file_name = split.last().unwrap();
    let mut file_name_split = file_name.split('.');
    file_name_split.next().unwrap().to_string()
}
