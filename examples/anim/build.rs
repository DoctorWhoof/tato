use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(), //
        force_reprocess: true,
    });

    let mut palette = PaletteBuilder::new("main");
    let mut groups = GroupBuilder::new();
    let mut bank = BankBuilder::new("BANK_ASTRO", &mut palette, &mut groups);
    bank.allow_unused = true;

    bank.new_animation_strip("import/astro.png", "STRIP_ASTRO", 8, 3);
    bank.write("src/astro.rs");
}
