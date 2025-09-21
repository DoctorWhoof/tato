use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(), //
        force_reprocess: false,
    });

    // Shared groups across tilesets
    let mut groups = GroupBuilder::new();

    // 9 Patch
    let mut palette_patch = PaletteBuilder::new("patch");
    let mut tileset_patch = TilesetBuilder::new("patch", &mut palette_patch, &mut groups);
    tileset_patch.new_map("import/patch.png", "PATCH_MAP");
    tileset_patch.write("src/patch.rs");

    // Smileys
    let mut palette_smileys = PaletteBuilder::new("smileys");
    let mut tileset_smileys = TilesetBuilder::new("smileys", &mut palette_smileys, &mut groups);
    tileset_smileys.new_map("import/smileys.png", "SMILEYS_MAP");
    tileset_smileys.write("src/smileys.rs");

    // Write groups to their own file
    groups.write("src/groups.rs");
}
