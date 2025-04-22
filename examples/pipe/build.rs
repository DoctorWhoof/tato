use tato_pipe::*;

fn main() {
    let mut pipe = Pipeline::new();

    // Define some groups that will be helpful during gameplay
    // Any inserted tiles can be flagged as part of a group
    // let grp_colliders = pipe.new_group("colliders");
    // let grp_items = pipe.new_group("items");

    // Create new empty palettes
    let palette_a = pipe.new_palette("fg_palette");

    // New empty tilesets, will populate their own tile pixels
    // and the colors on one of the palettes
    let tileset_a = pipe.new_tileset("gui", palette_a);

    // Finally, insert the actual assets insto a tileset
    // pipe.insert_anim("font.png", tileset_a, Some(grp_colliders));
    pipe.new_anim("assets/font.png", 1, 10, 4, tileset_a, palette_a);
    // A "font" is merely an animation where each frame is a letter!
    // pipe.insert_anim("font.png", tileset_a);

    pipe.write_assets("src/assets.rs");
}
