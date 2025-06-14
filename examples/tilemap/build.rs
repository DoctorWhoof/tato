use tato_pipe::*;

fn main() {
    let mut pipe = Pipeline::new();
    // pipe.allow_unused = true;

    // Tile Patch example
    let palette_patch = pipe.new_palette("patch");
    let tileset_patch = pipe.new_tileset("patch", palette_patch);

    pipe.new_map("../../assets/patch.png", tileset_patch);
    pipe.new_map("../../assets/default_tiles.png", tileset_patch);
    pipe.write_tileset(tileset_patch, "src/patch.rs");
}
