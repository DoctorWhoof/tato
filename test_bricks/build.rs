#[path ="src/specs.rs"]#[allow(unused)] mod specs;
use specs::*;
use tato_pipe::*;


fn main() {
    println!("cargo:warning=Running Build Script!");
    let mut atlas = AtlasBuilder::<TilesetID, PaletteID, GroupID>::new(SPECS);
    
    atlas.init_tileset( TilesetID::Hud, PaletteID::Main );
    atlas.init_font("assets/font.png", TilesetID::Hud, GroupID::None);

    atlas.init_tileset( TilesetID::Sprites, PaletteID::Main );
    atlas.init_anim("assets/hero.png", 10, 3, 2, TilesetID::Sprites, GroupID::None);
    atlas.init_anim("assets/puck.png", 10, 1, 1, TilesetID::Sprites, GroupID::None);
    
    atlas.init_tileset( TilesetID::Bg, PaletteID::Main );
    atlas.init_group("assets/groups/empty.png", TilesetID::Bg, GroupID::None, false);
    atlas.init_group("assets/groups/wall.png", TilesetID::Bg, GroupID::Wall, true);
    atlas.init_group("assets/groups/brick.png", TilesetID::Bg, GroupID::Brick, true);
    
    atlas.init_tilemap("assets/bg_01.png", TilesetID::Bg, GroupID::None); // tilemap 0

    atlas.save("assets/converted/atlas");
}