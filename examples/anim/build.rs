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

    bank.new_empty_tile();
    bank.new_animation_strip("import/astro.png", "STRIP_ASTRO", 8, 3);
    bank.new_anim("anim_down", "STRIP_ASTRO", 10, true, [4, 5, 6, 5]);
    bank.new_anim("anim_up", "STRIP_ASTRO", 10, true, [8, 9, 10, 9]);
    bank.new_anim("anim_right", "STRIP_ASTRO", 10, true, [12, 13, 14, 13]);

    bank.write("src/astro.rs");
}
