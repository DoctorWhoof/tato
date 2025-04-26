use crate::*;
// TODO: Add check for duplicated names

#[test]
fn test_anim() {
    let mut pipe = Pipeline::new();
    // Create new empty palettes
    let palette_fg = pipe.new_palette("palette_fg", 16);
    let palette_bg = pipe.new_palette("palette_bg", 16);

    // New empty tilesets, will populate their own tile pixels
    // and add colors to one of the palettes
    let tileset_gui = pipe.new_tileset("gui", palette_bg);
    let tileset_chars = pipe.new_tileset("chars", palette_fg);

    // A "font" is merely an animation where each frame is a letter!
    pipe.new_anim("test/font.png", 1, 10, 4, tileset_gui);
    pipe.new_anim("test/spy_idle.png", 8, 4, 1, tileset_chars);

    // Write output file
    // TODO: Maybe separate output file per tileset?
    pipe.write_assets("test/output.rs");
}
