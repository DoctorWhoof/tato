use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(), //
        force_reprocess: true,
    });

    // Shared groups across tilesets
    let mut groups = GroupBuilder::new();
    let mut palette = PaletteBuilder::new("patch");

    // 9 Patch
    let mut tileset_patch = TilesetBuilder::new("patch", &mut palette, &mut groups);
    tileset_patch.new_map("import/patch.png", "PATCH_MAP");
    tileset_patch.write("src/patch.rs");

    // Smileys
    let mut tileset_smileys = TilesetBuilder::new("smileys", &mut palette, &mut groups);
    tileset_smileys.new_map("import/smileys.png", "SMILEYS_MAP");
    tileset_smileys.write("src/smileys.rs");

    // Write groups to their own file
    groups.write("src/groups.rs");
}
