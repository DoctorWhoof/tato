#[path ="src/specs.rs"]#[allow(unused)] mod specs;
use specs::*;
use tato_pipe::*;

// TODO: Init assets based on Enum - all enum variants MUST be used!

fn main() {
    println!("cargo:warning=Running Build Script!");
    let mut atlas = AtlasBuilder::<TilesetID, PaletteID, GroupID>::new(SPECS);

    atlas.init_tileset( TilesetID::Player, PaletteID::Fg );
    atlas.init_anim("assets/bug_idle.png", 20, 2, 2, TilesetID::Player, GroupID::None);
    atlas.init_anim("assets/bug_shot.png", 10, 1, 1, TilesetID::Player, GroupID::None);
    
    atlas.init_tileset( TilesetID::Enemies, PaletteID::Fg );
    atlas.init_anim("assets/human_idle.png", 10, 2, 2, TilesetID::Enemies, GroupID::None);

    atlas.init_tileset( TilesetID::Bg, PaletteID::Bg );
    atlas.init_tilemap("assets/stars_bg.png", TilesetID::Bg, GroupID::None); // tilemap 0
    atlas.init_tilemap("assets/stars_fg.png", TilesetID::Bg, GroupID::None); // tilemap 1

    atlas.save("assets/converted/atlas");
}