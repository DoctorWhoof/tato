use tato_pipe::*;

fn main() {
    init_build(BuildSettings {
        asset_import_path: "import".into(),
        asset_export_path: "src/assets".into(),
        clear_export_path: true,
        force_reprocess: true,
    });

    let mut palette = PaletteBuilder::new("main");
    let mut bank = BankBuilder::new("ASTRO", &mut palette);
    bank.allow_unused = true;

    bank.new_empty_tile();
    bank.new_animation_strip("import/astro.png", "ASTRO", 8, 3);
    bank.new_anim("down", "ASTRO", 10, true, [4, 5, 6, 5]);
    bank.new_anim("up", "ASTRO", 10, true, [8, 9, 10, 9]);
    bank.new_anim("right", "ASTRO", 10, true, [12, 13, 14, 13]);

    bank.write("astro.rs");

    finalize_build();
}
