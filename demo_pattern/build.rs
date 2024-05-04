#[path ="src/specs.rs"]#[allow(unused)] mod specs;
use specs::*;
use tato_pipe::*;

fn main() {
    println!("cargo:warning=Running Build Script!");
    let mut atlas = AtlasBuilder::<TilesetID, PaletteID, GroupID>::new(SPECS);

    atlas.init_tileset( TilesetID::Bg, PaletteID::Bg );
    atlas.init_anim("assets/tile_A.png", 20, 1, 1, TilesetID::Bg, GroupID::None);   // 0
    atlas.init_anim("assets/tile_B.png", 20, 2, 2, TilesetID::Bg, GroupID::None);   // 1
    atlas.init_anim("assets/tile_D.png", 20, 1, 1, TilesetID::Bg, GroupID::None);   // 2
    
    atlas.save("assets/converted/atlas");
}