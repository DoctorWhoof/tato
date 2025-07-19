use tato::video::*;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("Generating RGBA12 palette with 512 colors...");

    // Create the .gpl file
    let mut file = File::create("examples/palette_save/rgb_512_colors.gpl").expect("Failed to create rgb_512_colors.gpl");

    // Write the header
    writeln!(file, "GIMP Palette").expect("Failed to write to file");
    writeln!(file, "Channels: RGBA").expect("Failed to write to file");
    writeln!(file, "#").expect("Failed to write to file");

    // Generate all RGB combinations (8^3 = 512) with full alpha (7 -> 255)
    let mut count = 0;
    for r in 0..8 {
        for g in 0..8 {
            for b in 0..8 {
                // Create RGBA12 color with full alpha (7)
                let color12 = RGBA12::new(r, g, b, 7);

                // Convert to RGBA32 to get the actual color values
                let color32 = RGBA32::from(color12);

                // Write the color to the file
                // Format: R G B A Name
                writeln!(file, "{:3} {:3} {:3} {:3} RGB_{}_{}_{}",
                         color32.r, color32.g, color32.b, color32.a,
                         r, g, b).expect("Failed to write color to file");

                count += 1;
            }
        }
    }

    println!("Successfully generated {} colors in rgba12_palette.gpl", count);
    println!("Each color uses 3 bits per RGB channel (0-7) with full alpha (255)");
    println!("Colors are named as RGB_r_g_b where r, g, b are the original 3-bit values");
    println!("File saved in current working directory: rgba12_palette.gpl");
}
