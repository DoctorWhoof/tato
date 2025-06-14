
const PROCESS_DEFAULT_FILES:bool = true;

fn main() {
    if !PROCESS_DEFAULT_FILES { return }

    use tato_pipe::*;
    let mut pipe = Pipeline::new();
    pipe.save_palettes = false;
    pipe.use_crate_assets = true; // Only true when used by this crate

    // Default font
    let palette_font = pipe.new_palette("font");
    let tileset_font = pipe.new_tileset("font", palette_font);

    pipe.disable_tile_transform_detection(tileset_font);
    // pipe.new_anim("assets/font_arcade_bold.png", 1, 10, 9, tileset_font);
    pipe.new_map("assets/font_arcade_bold.png", tileset_font);
    pipe.write_tileset(tileset_font, "src/tilesets/font_bold.rs");

    // Default basic tiles
    let palette_default = pipe.new_palette("default");
    let tileset_default = pipe.new_tileset("default", palette_default);

    // Checkers goes first to assure 4 colors in subpalette in right order
    pipe.new_tile("assets/tile_checkers.png", tileset_default);
    pipe.new_tile("assets/tile_empty.png", tileset_default);
    pipe.new_tile("assets/tile_solid.png", tileset_default);
    pipe.new_tile("assets/tile_crosshairs.png", tileset_default);
    pipe.new_tile("assets/tile_arrow.png", tileset_default);
    pipe.new_tile("assets/tile_smiley.png", tileset_default);
    // pipe.new_anim("assets/default_tiles.png", 1, 1, 1, tileset_default);

    pipe.write_tileset(tileset_default, "src/tilesets/default_tiles.rs");
}
