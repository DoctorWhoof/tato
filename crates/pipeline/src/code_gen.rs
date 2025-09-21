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
