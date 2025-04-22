use crate::*;

#[test]
fn test_anim(){
    let mut pipe = Pipeline::new();

    // Create new empty palettes
    let palette_a = pipe.new_palette("fg_palette");

    // New empty tilesets, will populate their own tile pixels
    // and the colors on one of the palettes
    let tileset_a = pipe.new_tileset("gui", palette_a);

    // Finally, insert the actual assets into a tileset
    // A "font" is merely an animation where each frame is a letter!
    pipe.new_anim("test/font.png", 1, 10, 4, tileset_a, palette_a);

    // Write output file
    pipe.write_assets("test/output.rs");
}
