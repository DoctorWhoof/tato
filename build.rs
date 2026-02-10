// This script converts PNG images into rust source code using
// Tato's data structs.
//
// In this case it generates the default assets, available to any
// project using Tato. Check out the "src/default_assets" folder
// to see the output.
fn main() {
    use tato_pipe::*;
    init_build(BuildSettings {
        asset_import_path: "import".into(),
        asset_export_path: "src/default_assets".into(),
        clear_export_path: true,
        force_reprocess: false,
    });

    // Default fonts
    let mut palette_font = PaletteBuilder::new("fonts");
    let mut bank_font_long = BankBuilder::new("FONT_LONG", &mut palette_font);
    bank_font_long.allow_unused = true;
    bank_font_long.use_crate_assets = true; // Only true when used by the main tato crate
    bank_font_long.new_map("font_long.png", "FONT_LONG");
    bank_font_long.write("font_long.rs");

    let mut bank_font_short = BankBuilder::new("FONT_SHORT", &mut palette_font);
    bank_font_short.allow_unused = true;
    bank_font_short.use_crate_assets = true;
    bank_font_short.new_map("font_short.png", "FONT_SHORT");
    bank_font_short.write("font_short.rs");

    let mut bank_font_arcade = BankBuilder::new("FONT_ARCADE", &mut palette_font);
    bank_font_arcade.allow_unused = true;
    bank_font_arcade.use_crate_assets = true;
    bank_font_arcade.new_map("font_arcade.png", "FONT_ARCADE");
    bank_font_arcade.write("font_arcade.rs");

    // Default basic tiles
    let mut palette_default = PaletteBuilder::new("default");
    let mut bank_default = BankBuilder::new("DEFAULT", &mut palette_default);
    bank_default.allow_unused = true;
    bank_default.use_crate_assets = true;
    // Add single tiles for default assets
    bank_default.new_tile("tile_empty.png");
    bank_default.new_tile("tile_checkers.png");
    bank_default.new_tile("tile_solid.png");
    bank_default.new_tile("tile_crosshairs.png");
    bank_default.new_tile("tile_arrow.png");
    bank_default.new_tile("tile_smiley.png");
    bank_default.write("default_tiles.rs");

    finalize_build();
}
