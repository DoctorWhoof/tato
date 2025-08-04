use tato_video::Cell;

use crate::builders::MapBuilder;
use std::fs::File;
use std::io::Write;

pub struct CodeWriter {
    output_file: File,
    indentation: usize,
}

// Note: indentation is now handled by simply calling rustfmt after generating code!
impl CodeWriter {
    pub fn new(path: &str) -> Self {
        let file = File::create(path).expect("Could not create output file");
        Self { output_file: file, indentation: 0 }
    }

    pub fn write_line(&mut self, line: &str) {
        let indent = " ".repeat(self.indentation);
        writeln!(self.output_file, "{}{}", indent, line).expect("Failed to write to output file");
    }

    pub fn write_header(&mut self, allow_unused: bool, use_crate_assets: bool) {
        // Removed timestamp to prevent too many unnecessary git changes
        // let timestamp = generate_timestamp();
        // self.write_line(&format!(
        //     "// Auto-generated code. Do not edit manually! Generated: {}",
        //     timestamp
        // ));
        self.write_line(&format!("// Auto-generated code. Do not edit manually!"));

        if allow_unused {
            self.write_line("#![allow(unused)]");
        }

        if use_crate_assets {
            self.write_line("use crate::prelude::*;");
        } else {
            self.write_line("use tato::prelude::*;");
        }

        self.write_line("");
        self.write_line("");
    }

    pub fn write_tileset_data_struct(
        &mut self,
        name: &str,
        save_colors: bool,
        sub_palette_count: usize,
    ) {
        self.write_line(&format!(
            "pub const {}_TILESET: TilesetData = TilesetData{{",
            name.to_uppercase(),
        ));
        self.write_line(&format!("    tiles: &{}_TILES,", name.to_uppercase()));

        if save_colors {
            self.write_line(&format!("    colors: Some(&{}_COLORS),", name.to_uppercase()));
        } else {
            self.write_line("    colors: None,");
        }

        if save_colors && sub_palette_count > 0 {
            self.write_line(&format!("    sub_palettes: Some(&["));
            for i in 0..sub_palette_count {
                self.write_line(&format!("        &{}_SUBPALETTE_{},", name.to_uppercase(), i));
            }
            self.write_line("    ]),");
        } else {
            self.write_line("    sub_palettes: None,");
        }

        self.write_line("};");
        self.write_line("");
    }

    pub fn write_color_array(&mut self, name: &str, colors: &[tato_video::RGBA12]) {
        if colors.is_empty() {
            return;
        }

        self.write_line(&format!(
            "pub const {}_COLORS: [RGBA12; {}] = [",
            name.to_uppercase(),
            colors.len()
        ));

        for color in colors {
            self.write_line(&format!(
                "    RGBA12::new({}, {}, {}, {}),",
                color.r(),
                color.g(),
                color.b(),
                color.a()
            ));
        }

        self.write_line("];");
        self.write_line("");
    }

    pub fn write_sub_palette(&mut self, name: &str, index: usize, palette: &[u8]) {
        self.write_line(&format!(
            "pub const {}_SUBPALETTE_{}: [u8; {}] = [",
            name.to_uppercase(),
            index,
            palette.len()
        ));

        for &color_index in palette {
            self.write_line(&format!("    {},", color_index));
        }

        self.write_line("];");
        self.write_line("");
    }

    pub fn write_tilemap_constant(
        &mut self,
        name: &str,
        columns: u8,
        rows: u8,
        cells: &[tato_video::Cell],
    ) {
        self.write_line(&format!(
            "pub const {}: Tilemap<{}> = Tilemap {{",
            name.to_uppercase(),
            cells.len()
        ));
        self.write_line(&format!("    columns: {},", columns));
        self.write_line(&format!("    rows: {},", rows));
        self.write_line("    cells: [");

        for cell in cells {
            self.write_line(&format!("        {:?},", cell));
        }

        self.write_line("    ],");
        self.write_line("};");
        self.write_line("");
    }

    pub fn write_tile_id_constant(&mut self, name: &str, id: u8) {
        self.write_line(&format!("pub const {}: TileID = TileID({});", name.to_uppercase(), id));
    }

    pub fn write_cell_constant(&mut self, name: &str, cell: Cell) {
        self.write_line(&format!(
            "pub const {}: Cell = {:?};",
            name.to_uppercase(),
            cell
        ));
    }

    pub fn write_group_constant(&mut self, name: &str, group_index: u8) {
        let group_value = 1u16 << (group_index - 1); // Convert 1-based index to bit value
        self.write_line(&format!("pub const {}: u16 = {};", name.to_uppercase(), group_value));
    }

    pub fn write_tile_array_header(&mut self, name: &str, tile_count: usize) {
        self.write_line(&format!(
            "pub const {}_TILES: [Tile<2>; {}] = [",
            name.to_uppercase(),
            tile_count
        ));
    }

    pub fn write_tile_cluster(&mut self, byte0: u8, byte1: u8) {
        self.write_line(&format!("            Cluster {{ data: [{}, {}] }},", byte0, byte1));
    }

    pub fn write_animation_strip(&mut self, name: &str, frames: &[MapBuilder]) {
        self.write_line(&format!(
            "pub const {}: [Tilemap<{}>; {}] = [",
            name.to_uppercase(),
            frames[0].cells.len(),
            frames.len()
        ));

        for frame in frames {
            self.write_line("    Tilemap {");
            self.write_line("        cells: [");

            for cell in &frame.cells {
                self.write_line(&format!("            {:?},", cell));
            }

            self.write_line("        ],");
            self.write_line(&format!("        columns: {},", frame.columns));
            self.write_line(&format!("        rows: {},", frame.rows));
            self.write_line("    },");
        }

        self.write_line("];");
        self.write_line("");
    }

    pub fn format_output(&self, file_path: &str) {
        use std::process::Command;
        let output = Command::new("rustfmt").arg(file_path).output();
        if let Err(e) = output {
            println!("cargo:warning=rustfmt failed: {}", e);
        }
    }
}

// /// Generates a timestamp string for code generation headers
// pub fn generate_timestamp() -> String {
//     use std::process::Command;

//     // Try to get local date and time using system date command
//     if let Ok(output) = Command::new("date").arg("+%Y-%m-%d %H:%M:%S").output() {
//         if output.status.success() {
//             if let Ok(timestamp) = String::from_utf8(output.stdout) {
//                 return timestamp.trim().to_string();
//             }
//         }
//     }

//     // Fallback to basic UTC time if system date command fails
//     std::time::SystemTime::now()
//         .duration_since(std::time::UNIX_EPOCH)
//         .map(|duration| {
//             let seconds = duration.as_secs();
//             let hours = (seconds / 3600) % 24;
//             let minutes = (seconds / 60) % 60;
//             let secs = seconds % 60;
//             format!("{:02}:{:02}:{:02} UTC", hours, minutes, secs)
//         })
//         .unwrap_or_else(|_| "build time".to_string())
// }
