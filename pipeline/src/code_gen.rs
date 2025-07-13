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
        Self {
            output_file: file,
            indentation: 0,
        }
    }

    pub fn write_line(&mut self, line: &str) {
        let indent = " ".repeat(self.indentation);
        writeln!(self.output_file, "{}{}", indent, line).expect("Failed to write to output file");
    }

    pub fn write(&mut self, line: &str) {
        write!(self.output_file, "{}", line).expect("Failed to write to output file");
    }

    // pub fn start_line(&mut self, line: &str) {
    //     let indent = " ".repeat(self.indentation);
    //     write!(self.output_file, "{}{}", indent, line).expect("Failed to write to output file");
    // }

    // pub fn finish_line(&mut self, line: &str) {
    //     write!(self.output_file, "{}", line).expect("Failed to write to output file");
    // }
}
