use tato_pipe::*;

fn main() {
    let mut pipe = Pipeline::new();
    let palette = pipe.new_palette("font", 16);
    let tileset = pipe.new_tileset("font", palette);

    pipe.disable_tile_transform_detection(tileset);
    pipe.new_anim("assets/font_arcade_bold.png", 1, 10, 9, tileset);
    pipe.write_tileset_pixels("src/fonts/font_bold.rs");
}
