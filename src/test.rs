use crate::*;

#[test]
fn test_anim() {
    let mut pipe = Pipeline::new();
    // TODO: Add color limits for palette and subpalette on pipeline
    // TODO: Add check for duplicated names
    // Create new empty palettes
    let palette_fg = pipe.new_palette("palette_fg", 16);

    // New empty tilesets, will populate their own tile pixels
    // and the colors on one of the palettes
    let tileset_a = pipe.new_tileset("gui");

    // Finally, insert the actual assets into a tileset
    // A "font" is merely an animation where each frame is a letter!
    pipe.new_anim("test/font.png", 1, 10, 4, tileset_a, palette_fg);
    pipe.new_anim("test/spy_idle.png", 8, 4, 1, tileset_a, palette_fg);

    // Write output file
    pipe.write_assets("test/output.rs");
}
