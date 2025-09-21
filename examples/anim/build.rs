use tato_pipe::*;

fn main() {
    init_build("import");

    let mut palette = PaletteBuilder::new("main");
    let mut groups = GroupBuilder::new();
    let mut tileset = TilesetBuilder::new("astro", &mut palette, &mut groups);
    tileset.allow_unused = true;

    tileset.new_animation_strip("import/astro.png", "STRIP_ASTRO", 8, 3);
    tileset.write("src/astro.rs");
}
