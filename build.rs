const REGENERATE_DEFAULT_ASSETS: bool = false;

fn main() {
    if !REGENERATE_DEFAULT_ASSETS {
        return;
    }
    use tato_pipe::*;
    let mut pipe = Pipeline::new();
    pipe.allow_unused = true;
    pipe.save_palettes = false; // Sticks with default palette
    pipe.use_crate_assets = true; // Only true when used by this crate

    // Default font
    let palette_font = pipe.new_palette("font");
    let tileset_font = pipe.new_tileset("font", palette_font);
    pipe.new_map("assets/font_bold.png", "FONT_MAP", tileset_font);
    pipe.write_tileset(tileset_font, "src/default_assets/font_bold.rs");

    // Default basic tiles
    let palette_default = pipe.new_palette("default");
    let tileset_default = pipe.new_tileset("default", palette_default);
    pipe.new_tile("assets/tile_empty.png", tileset_default);
    // Checkers goes first to assure 4 colors in subpalette in the desired order
    pipe.new_tile("assets/tile_checkers.png", tileset_default);
    pipe.new_tile("assets/tile_solid.png", tileset_default);
    pipe.new_tile("assets/tile_crosshairs.png", tileset_default);
    pipe.new_tile("assets/tile_arrow.png", tileset_default);
    pipe.new_tile("assets/tile_smiley.png", tileset_default);
    pipe.write_tileset(tileset_default, "src/default_assets/default_tiles.rs");
}
