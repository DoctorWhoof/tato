use std::fs::File;
use std::io::Write;

pub struct CodeWriter {
    output_file: File,
    indentation: usize,
}

// Code generation now creates `static` items with automatic platform-specific `link_section` attributes
// for optimal bare-metal/embedded usage:
//
// - `static` provides stable memory addresses and single instance in memory
// - `link_section` allows placement in read-only sections that stay in flash/ROM
// - Automatically uses ".rodata" for ELF targets (Linux, embedded)
// - Automatically uses "__DATA,__const" for macOS (Mach-O format)
// - Automatically uses ".rdata" for Windows (PE/COFF format)
// - Data is loaded on-demand rather than eagerly into RAM
//
// Note: indentation is now handled by simply calling rustfmt after generating code!

/// Returns the appropriate link section for the current platform
pub fn get_platform_link_section() -> &'static str {
    if cfg!(target_os = "macos") {
        "__DATA,__const"  // Mach-O format
    } else if cfg!(target_os = "windows") {
        ".rdata"          // PE/COFF read-only data section
    } else {
        ".rodata"         // ELF format (Linux, embedded)
    }
}

/// Formats a Cell using the compact Cell::new() constructor syntax
pub fn format_cell_compact(cell: &tato_video::Cell) -> String {
    format!(
        "Cell::new({}, {}, {}, {})",
        cell.id.0,
        cell.flags.0,
        cell.color_mapping,
        cell.group
    )
}

/// Formats a Tile using the compact Tile::new(u64, u64) constructor syntax
pub fn format_tile_compact(tile_pixels: &[u8]) -> String {
    assert_eq!(tile_pixels.len(), 64, "Tile must have exactly 64 pixels");

    let mut data0 = 0u64;
    let mut data1 = 0u64;

    // Process first 4 clusters (rows 0-3) into data0
    for cluster_idx in 0..4 {
        for byte_idx in 0..2 {
            let mut byte_val = 0u8;
            // Pack 4 pixels (2 bits each) into one byte
            for pixel_in_byte in 0..4 {
                let pixel_idx = cluster_idx * 8 + byte_idx * 4 + pixel_in_byte;
                let pixel_val = tile_pixels[pixel_idx] & 0x3; // Ensure 2-bit pixel
                byte_val |= pixel_val << (6 - pixel_in_byte * 2);
            }
            let shift = (7 - (cluster_idx * 2 + byte_idx)) * 8;
            data0 |= (byte_val as u64) << shift;
        }
    }

    // Process second 4 clusters (rows 4-7) into data1
    for cluster_idx in 0..4 {
        for byte_idx in 0..2 {
            let mut byte_val = 0u8;
            // Pack 4 pixels (2 bits each) into one byte
            for pixel_in_byte in 0..4 {
                let pixel_idx = (cluster_idx + 4) * 8 + byte_idx * 4 + pixel_in_byte;
                let pixel_val = tile_pixels[pixel_idx] & 0x3; // Ensure 2-bit pixel
                byte_val |= pixel_val << (6 - pixel_in_byte * 2);
            }
            let shift = (7 - (cluster_idx * 2 + byte_idx)) * 8;
            data1 |= (byte_val as u64) << shift;
        }
    }

    format!("Tile::new(0x{:016X}, 0x{:016X})", data0, data1)
}

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

    pub fn write_color_array(&mut self, name: &str, colors: &[tato_video::RGBA12]) {
        if colors.is_empty() {
            return;
        }

        // Use platform-specific link section for optimal bare-metal usage
        self.write_line(&format!("#[unsafe(link_section = \"{}\")]", get_platform_link_section()));
        self.write_line(&format!(
            "pub static {}_COLORS: [RGBA12; {}] = [",
            name.to_uppercase(),
            colors.len()
        ));

        for color in colors {
            self.write_line(&format!(
                "    RGBA12::with_transparency({}, {}, {}, {}),",
                color.r(),
                color.g(),
                color.b(),
                color.a()
            ));
        }

        self.write_line("];");
        self.write_line("");
    }

    pub fn write_group_constant(&mut self, name: &str, group_index: u8) {
        let group_value = 1u16 << (group_index - 1); // Convert 1-based index to bit value
        self.write_line(&format!("pub const {}: u8 = {};", name.to_uppercase(), group_value));
    }

    pub fn write_cell(&mut self, cell: &tato_video::Cell) {
        self.write_line(&format!("        {},", format_cell_compact(cell)));
    }

    pub fn format_output(&self, file_path: &str) {
        use std::process::Command;
        let output = Command::new("rustfmt").arg(file_path).output();
        if let Err(e) = output {
            println!("cargo:warning=rustfmt failed: {}", e);
        }
    }
}

// Removed timestamp to prevent too many unnecessary git changes
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
