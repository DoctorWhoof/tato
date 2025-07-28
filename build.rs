const REGENERATE_DEFAULT_ASSETS: bool = false;

fn main() {
    if !REGENERATE_DEFAULT_ASSETS {
        return;
    }
    use tato_pipe::*;
    init_build();

    // Default font
    let mut palette_font = PaletteBuilder::new("font");
    let mut tileset_font = TilesetBuilder::new("font", &mut palette_font);
    tileset_font.allow_unused = true;
    tileset_font.save_colors = false; // Sticks with default palette
    tileset_font.use_crate_assets = true; // Only true when used by this crate
    tileset_font.new_map("import/font_bold.png", "FONT_MAP");
    tileset_font.write("src/default_assets/font_bold.rs");

    // Default basic tiles
    let mut palette_default = PaletteBuilder::new("default");
    let mut tileset_default = TilesetBuilder::new("default", &mut palette_default);
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
}
