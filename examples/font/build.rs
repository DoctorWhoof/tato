use tato_pipe::*;

fn main() {
    let mut pipe = Pipeline::new();
    pipe.allow_unused = true;
    pipe.save_palettes = false;

    // Tile Patch example
    let palette_font = pipe.new_palette("font");
    let tileset_font = pipe.new_tileset("font", palette_font);

    pipe.new_map("../../assets/font_arcade_bold.png", tileset_font); // for debugging
    pipe.write_tileset(tileset_font, "src/font.rs");
}
