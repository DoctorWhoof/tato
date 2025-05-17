
fn main() {
    use tato::pipe::*;
    // Print working directory
    println!("Current working directory: {:?}", std::env::current_dir().unwrap());

    let mut pipe = Pipeline::new();

    // Tile Patch example
    let palette_patch = pipe.new_palette("patch", 16);
    let tileset_patch = pipe.new_tileset("patch", palette_patch);

    pipe.new_anim("../../assets/patch.png", 1, 1, 1, tileset_patch);
    pipe.write_tileset(tileset_patch, "src/patch.rs");
}
