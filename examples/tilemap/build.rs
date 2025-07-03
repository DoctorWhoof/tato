use tato_pipe::*;

fn main() {
    let mut pipe = Pipeline::new();

    // 9 Patch
    let palette_patch = pipe.new_palette("patch");
    let tileset_patch = pipe.new_tileset("patch", palette_patch);
    pipe.new_map("../../assets/patch.png", tileset_patch);
    pipe.write_tileset(tileset_patch, "src/patch.rs");

    // Smileys
    let palette_smileys = pipe.new_palette("smileys");
    let tileset_smileys = pipe.new_tileset("smileys", palette_smileys);
    pipe.new_map("assets/smileys.png", tileset_smileys);
    pipe.write_tileset(tileset_smileys, "src/smileys.rs");

    // Test
    let palette_test = pipe.new_palette("testmap");
    let tileset_test = pipe.new_tileset("testmap", palette_test);
    pipe.new_map("assets/test.png", tileset_test);
    pipe.write_tileset(tileset_test, "src/testmap.rs");
}
