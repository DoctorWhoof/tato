use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(),
        force_reprocess: true,
    });

    // Shared groups across banks
    let mut groups = GroupBuilder::new();
    let mut palette = PaletteBuilder::new("patch");

    // 9 Patch
    let mut bank_patch = BankBuilder::new("PATCH", &mut palette, &mut groups);
    bank_patch.new_map("import/patch.png", "PATCH");
    bank_patch.write("src/patch.rs");

    // Smileys
    let mut bank_smileys = BankBuilder::new("SMILEYS", &mut palette, &mut groups);
    bank_smileys.new_map("import/smileys.png", "SMILEYS");
    bank_smileys.write("src/smileys.rs");

    // Write groups to their own file
    groups.write("src/groups.rs");
}
