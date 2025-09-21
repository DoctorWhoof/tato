// This script converts PNG images into rust source code using
// Tato's data structs.
//
// In this case it generates the default assets, available to any
// project using Tato. Check out the "src/default_assets" folder
// to see the output.
//
// It is disabled by default to speed up build times.
fn main() {
    use tato_pipe::*;
    init_build("import", false);

    // Shared groups for default assets
    let mut groups = GroupBuilder::new();

    // Default fonts
    let mut palette_font = PaletteBuilder::new("fonts");
    let mut tileset_font = TilesetBuilder::new("font_long", &mut palette_font, &mut groups);
    tileset_font.allow_unused = true;
    tileset_font.save_colors = false; // Sticks with default palette
    tileset_font.use_crate_assets = true; // Only true when used by this crate
    tileset_font.new_map("import/font_long.png", "FONT_LONG_MAP");
    tileset_font.write("src/default_assets/font_long.rs");

    let mut tileset_short = TilesetBuilder::new("font_short", &mut palette_font, &mut groups);
    tileset_short.allow_unused = true;
    tileset_short.save_colors = false; // Sticks with default palette
    tileset_short.use_crate_assets = true; // Only true when used by this crate
    tileset_short.new_map("import/font_short.png", "FONT_SHORT_MAP");
    tileset_short.write("src/default_assets/font_short.rs");

    let mut tileset_arcade = TilesetBuilder::new("font_arcade", &mut palette_font, &mut groups);
    tileset_arcade.allow_unused = true;
    tileset_arcade.save_colors = false; // Sticks with default palette
    tileset_arcade.use_crate_assets = true; // Only true when used by this crate
    tileset_arcade.new_map("import/font_arcade.png", "FONT_ARCADE_MAP");
    tileset_arcade.write("src/default_assets/font_arcade.rs");

    // Default basic tiles
    let mut palette_default = PaletteBuilder::new("default");
    let mut tileset_default = TilesetBuilder::new("default", &mut palette_default, &mut groups);
    tileset_default.allow_unused = true;
    tileset_default.save_colors = false; // Sticks with default palette
    tileset_default.use_crate_assets = true; // Only true when used by this crate
    // Add single tiles for default assets
    tileset_default.new_tile("import/tile_empty.png");
    // Checkers goes first to assure 4 colors in subpalette in the desired order
    tileset_default.new_tile("import/tile_checkers.png");
    tileset_default.new_tile("import/tile_solid.png");
    tileset_default.new_tile("import/tile_crosshairs.png");
    tileset_default.new_tile("import/tile_arrow.png");
    tileset_default.new_tile("import/tile_smiley.png");
    tileset_default.write("src/default_assets/default_tiles.rs");

    // Write groups to their own file (if any)
    groups.write("src/default_assets/groups.rs");
}
