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
        force_reprocess: true,
    });

    // Shared groups for default assets
    let mut groups = GroupBuilder::new();

    // Default fonts
    let mut palette_font = PaletteBuilder::new("fonts");
    let mut bank_font_long = BankBuilder::new("BANK_FONT_LONG", &mut palette_font, &mut groups);
    bank_font_long.allow_unused = true;
    bank_font_long.use_crate_assets = true; // Only true when used by this crate
    bank_font_long.new_map("import/font_long.png", "FONT_LONG_MAP");
    bank_font_long.write("src/default_assets/font_long.rs");

    let mut bank_font_short = BankBuilder::new("BANK_FONT_SHORT", &mut palette_font, &mut groups);
    bank_font_short.allow_unused = true;
    bank_font_short.use_crate_assets = true;
    bank_font_short.new_map("import/font_short.png", "FONT_SHORT_MAP");
    bank_font_short.write("src/default_assets/font_short.rs");

    let mut bank_font_arcade = BankBuilder::new("BANK_FONT_ARCADE", &mut palette_font, &mut groups);
    bank_font_arcade.allow_unused = true;
    bank_font_arcade.use_crate_assets = true;
    bank_font_arcade.new_map("import/font_arcade.png", "FONT_ARCADE_MAP");
    bank_font_arcade.write("src/default_assets/font_arcade.rs");

    // Default basic tiles
    let mut palette_default = PaletteBuilder::new("default");
    let mut bank_default = BankBuilder::new("BANK_DEFAULT", &mut palette_default, &mut groups);
    bank_default.allow_unused = true;
    bank_default.use_crate_assets = true;
    // Add single tiles for default assets
    bank_default.new_tile("import/tile_empty.png");
    bank_default.new_tile("import/tile_checkers.png");
    bank_default.new_tile("import/tile_solid.png");
    bank_default.new_tile("import/tile_crosshairs.png");
    bank_default.new_tile("import/tile_arrow.png");
    bank_default.new_tile("import/tile_smiley.png");
    bank_default.write("src/default_assets/default_tiles.rs");

    // Write groups to their own file (if any)
    groups.write("src/default_assets/groups.rs");
}
