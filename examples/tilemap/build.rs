use tato_pipe::*;

fn main() {
    init_build();

    // 9 Patch
    let mut palette_patch = PaletteBuilder::new("patch");
    let mut tileset_patch = TilesetBuilder::new("patch", &mut palette_patch);
    tileset_patch.new_map("import/patch.png", "PATCH_MAP");
    tileset_patch.write("src/patch.rs");

    // Smileys
    let mut palette_smileys = PaletteBuilder::new("smileys");
    let mut tileset_smileys = TilesetBuilder::new("smileys", &mut palette_smileys);
    tileset_smileys.new_map("import/smileys.png", "SMILEYS_MAP");
    tileset_smileys.write("src/smileys.rs");
}
