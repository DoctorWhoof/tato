use crate::*;
// TODO: Add check for duplicated names

#[test]
fn test_anim() {
    let mut pipe = Pipeline::new();
    // Create new empty palettes
    let palette_fg = pipe.new_palette("palette_fg");
    let palette_bg = pipe.new_palette("palette_bg");

    // New empty tilesets, will populate their own tile pixels
    // and add colors to one of the palettes
    let tileset_chars = pipe.new_tileset("chars", palette_fg);

    // Tile transforms (flip, rotate) can mess up font indices, so
    // let's disable them in these tilesets.
    let tileset_font = pipe.new_tileset("font_simple", palette_bg);
    pipe.disable_tile_transform_detection(tileset_font);

    // Write output file
    // TODO: Maybe separate output file per tileset?
    pipe.write_tileset(tileset_chars, "test/output.rs");
}
