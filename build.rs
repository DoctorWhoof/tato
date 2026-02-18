// This script converts PNG images into rust source code using
// Tato's data structs.
//
// In this case it generates the default assets, available to any
// project using Tato. Check out the "src/default_assets" folder
// to see the output.
//
const SKIP: bool = true;
fn main() {
    if SKIP {
        return;
    }

    use tato_pipe::*;
    init_build(BuildSettings {
        asset_import_path: "import".into(),
        asset_export_path: "src/default_assets".into(),
        clear_export_path: true,
        force_reprocess: true,
    });

    // Default tiles
    let mut palette = PaletteBuilder::new("DEFAULT");

    let mut bank_dither = BankBuilder::new("DITHER", &mut palette);
    bank_dither.use_crate_assets = true; // Only true when used by the main tato crate
    bank_dither.allow_unused = true;
    bank_dither.write_tiles = true;
    bank_dither.write_animations = false;
    bank_dither.write_colors = false;
    bank_dither.write_maps = false;
    bank_dither.new_strip("dither.png", "dither", 9, 1);
    bank_dither.write("dither.rs");

    let mut bank_lines = BankBuilder::new("LINES", &mut palette);
    bank_lines.use_crate_assets = true; // Only true when used by the main tato crate
    bank_lines.allow_unused = true;
    bank_lines.write_tiles = true;
    bank_lines.write_animations = false;
    bank_lines.write_colors = false;
    bank_lines.write_maps = false;
    bank_lines.new_strip("lines.png", "lines", 7, 1);
    bank_lines.write("lines.rs");

    let mut bank_chars = BankBuilder::new("CHARS", &mut palette);
    bank_chars.use_crate_assets = true; // Only true when used by the main tato crate
    bank_chars.allow_unused = true;
    bank_chars.write_tiles = true;
    bank_chars.write_animations = false;
    bank_chars.write_colors = false;
    bank_chars.write_maps = true;
    bank_chars.new_strip("chars.png", "chars", 16, 5);
    bank_chars.new_map("font_long.png", "FONT_LONG");
    bank_chars.write("chars.rs");

    let mut bank_icons = BankBuilder::new("ICONS", &mut palette);
    bank_icons.use_crate_assets = true; // Only true when used by the main tato crate
    bank_icons.allow_unused = true;
    bank_icons.write_tiles = true;
    bank_icons.write_animations = false;
    bank_icons.write_colors = false;
    bank_icons.write_maps = false;
    bank_icons.new_strip("icons.png", "icons", 12, 1);
    bank_icons.write("icons.rs");

    let mut bank_lines = BankBuilder::new("LINES", &mut palette);
    bank_lines.use_crate_assets = true; // Only true when used by the main tato crate
    bank_lines.allow_unused = true;
    bank_lines.write_tiles = true;
    bank_lines.write_animations = false;
    bank_lines.write_colors = false;
    bank_lines.write_maps = false;
    bank_lines.new_strip("lines.png", "lines", 11, 5);
    bank_lines.write("lines.rs");

    let mut bank_diagonals = BankBuilder::new("DIAGONALS", &mut palette);
    bank_diagonals.use_crate_assets = true; // Only true when used by the main tato crate
    bank_diagonals.allow_unused = true;
    bank_diagonals.write_tiles = true;
    bank_diagonals.write_animations = false;
    bank_diagonals.write_colors = false;
    bank_diagonals.write_maps = false;
    bank_diagonals.new_strip("diagonals.png", "diagonals", 16, 1);
    bank_diagonals.write("diagonals.rs");

    let mut bank_surfaces = BankBuilder::new("SURFACES", &mut palette);
    bank_surfaces.use_crate_assets = true; // Only true when used by the main tato crate
    bank_surfaces.allow_unused = true;
    bank_surfaces.write_tiles = true;
    bank_surfaces.write_animations = false;
    bank_surfaces.write_colors = false;
    bank_surfaces.write_maps = false;
    bank_surfaces.new_strip("surfaces.png", "surfaces", 16, 1);
    bank_surfaces.write("surfaces.rs");

    let mut bank_symbols = BankBuilder::new("SYMBOLS", &mut palette);
    bank_symbols.use_crate_assets = true; // Only true when used by the main tato crate
    bank_symbols.allow_unused = true;
    bank_symbols.write_tiles = true;
    bank_symbols.write_animations = false;
    bank_symbols.write_colors = false;
    bank_symbols.write_maps = false;
    bank_symbols.new_strip("symbols.png", "symbols", 16, 1);
    bank_symbols.write("symbols.rs");

    let mut bank_frames = BankBuilder::new("FRAMES", &mut palette);
    bank_frames.use_crate_assets = true; // Only true when used by the main tato crate
    bank_frames.allow_unused = true;
    bank_frames.write_tiles = true;
    bank_frames.write_animations = false;
    bank_frames.write_colors = false;
    bank_frames.write_maps = false;
    bank_frames.new_strip("frames.png", "frames", 16, 1);
    bank_frames.write("frames.rs");

    let mut bank_circles = BankBuilder::new("CIRCLES", &mut palette);
    bank_circles.use_crate_assets = true; // Only true when used by the main tato crate
    bank_circles.allow_unused = true;
    bank_circles.write_tiles = true;
    bank_circles.write_animations = false;
    bank_circles.write_colors = false;
    bank_circles.write_maps = false;
    bank_circles.new_strip("circles.png", "circles", 16, 1);
    bank_circles.write("circles.rs");

    let mut bank_grids = BankBuilder::new("GRIDS", &mut palette);
    bank_grids.use_crate_assets = true; // Only true when used by the main tato crate
    bank_grids.allow_unused = true;
    bank_grids.write_tiles = true;
    bank_grids.write_animations = false;
    bank_grids.write_colors = false;
    bank_grids.write_maps = false;
    bank_grids.new_strip("grids.png", "grids", 16, 1);
    bank_grids.write("grids.rs");

    let mut bank_misc = BankBuilder::new("MISC", &mut palette);
    bank_misc.use_crate_assets = true; // Only true when used by the main tato crate
    bank_misc.allow_unused = true;
    bank_misc.write_tiles = true;
    bank_misc.write_animations = false;
    bank_misc.write_colors = false;
    bank_misc.write_maps = false;
    bank_misc.new_strip("misc.png", "misc", 16, 1);
    bank_misc.write("misc.rs");

    finalize_build();
}
