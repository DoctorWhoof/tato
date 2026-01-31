use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(),
        asset_export_path: "src/assets".into(),
        clear_export_path: false,
        force_reprocess: true,
    });

    // Shared across banks
    let mut palette = PaletteBuilder::new("patch");

    // 9 Patch
    let mut bank_patch = BankBuilder::new("PATCH", &mut palette);
    bank_patch.new_map("import/patch.png", "PATCH");
    bank_patch.write("patch.rs");

    // // Smileys
    let mut bank_smileys = BankBuilder::new("SMILEYS", &mut palette);
    bank_smileys.new_map("import/smileys.png", "SMILEYS");
    bank_smileys.write("smileys.rs");

    finalize_build();
}
